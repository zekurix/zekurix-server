#!/usr/bin/env python3
"""Send HTTP requests to the Zekurix API, optionally authenticated as a mock user.

Supported curl-like arguments:
URL
-X/--request METHOD
-H/--header HEADER
-d/--data DATA

Examples:
python tests/scripts/api-request.py http://localhost:3000/health

python tests/scripts/api-request.py \
    -u Alice \
    http://localhost:3000/api/v1/users/me

python tests/scripts/api-request.py \
    -u admin \
    -X POST \
    -H "Content-Type: application/json" \
    -d '{"identity":{"issuer":"http://localhost:3001/my-issuer","subject":"Alice"}}' \
    http://localhost:3000/api/v1/users
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import tomllib
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass, field
from pathlib import Path

CONFIG_PATH_ENV = "ZEKURIX_CONFIG_PATH"
ISSUER_ENV = "ZEKURIX_AUTHENTICATION__ISSUER"
PROJECT_ROOT = Path(__file__).resolve().parents[2]
ENV_FILE = PROJECT_ROOT / ".env"
CLIENT_ID = "zekurix-test-api-request"
CLIENT_SECRET = "zekurix-test-api-request-secret"
TIMEOUT = 10.0


@dataclass
class RequestArguments:
    url: str
    method: str | None = None
    headers: list[str] = field(default_factory=list)
    data: list[str] = field(default_factory=list)


def parse_cli() -> tuple[argparse.Namespace, RequestArguments]:
    parser = argparse.ArgumentParser(
        description="Send an HTTP request to Zekurix, optionally authenticated as a mock user.",
        allow_abbrev=False,
    )
    parser.add_argument("-u", "--user", help="Username used as JWT subject")

    args, remaining = parser.parse_known_args()
    request = parse_request_arguments(remaining, parser)
    return args, request


def parse_request_arguments(
    arguments: list[str], parser: argparse.ArgumentParser
) -> RequestArguments:
    method: str | None = None
    headers: list[str] = []
    data: list[str] = []
    url: str | None = None

    index = 0
    while index < len(arguments):
        argument = arguments[index]

        if argument == "--":
            index += 1
            continue

        if argument in ("-X", "--request"):
            index += 1
            if index >= len(arguments):
                parser.error(f"{argument} requires a method")
            method = arguments[index].upper()
        elif argument.startswith("--request="):
            method = argument.split("=", 1)[1].upper()
        elif argument in ("-H", "--header"):
            index += 1
            if index >= len(arguments):
                parser.error(f"{argument} requires a header")
            headers.append(arguments[index])
        elif argument.startswith("--header="):
            headers.append(argument.split("=", 1)[1])
        elif argument in ("-d", "--data", "--data-raw"):
            index += 1
            if index >= len(arguments):
                parser.error(f"{argument} requires data")
            data.append(arguments[index])
        elif argument.startswith(("--data=", "--data-raw=")):
            data.append(argument.split("=", 1)[1])
        elif argument.startswith("-"):
            parser.error(f"unsupported curl option: {argument}")
        elif url is None:
            url = argument
        else:
            parser.error(f"unexpected argument: {argument}")

        index += 1

    if url is None:
        parser.error("the target URL is required")

    return RequestArguments(url=url, method=method, headers=headers, data=data)


def load_dotenv_value(name: str) -> str | None:
    if not ENV_FILE.exists():
        return None

    for line in ENV_FILE.read_text(encoding="utf-8").splitlines():
        line = line.strip()

        if not line or line.startswith("#"):
            continue

        key, separator, value = line.partition("=")
        if separator and key.strip() == name:
            return value.strip().strip("\"'") or None

    return None


def get_config_file() -> Path | None:
    config_path = os.getenv(CONFIG_PATH_ENV) or load_dotenv_value(CONFIG_PATH_ENV)

    if not config_path:
        return None

    return PROJECT_ROOT / config_path


def load_config_issuer() -> str | None:
    config_file = get_config_file()

    if config_file is None or not config_file.exists():
        return None

    with config_file.open("rb") as file:
        config = tomllib.load(file)

    return config.get("authentication", {}).get("issuer")


def get_issuer() -> str:
    issuer = (
        os.getenv(ISSUER_ENV) or load_dotenv_value(ISSUER_ENV) or load_config_issuer()
    )

    if not issuer:
        raise RuntimeError(
            f"{ISSUER_ENV} must be defined in the environment, "
            f"{ENV_FILE}, or authentication.issuer in the configured TOML file"
        )

    return issuer


def get_token_url() -> str:
    return f"{get_issuer().rstrip('/')}/token"


def request_json(url: str, form: dict[str, str]) -> dict:
    body = urllib.parse.urlencode(form).encode("utf-8")
    request = urllib.request.Request(
        url,
        data=body,
        headers={"Content-Type": "application/x-www-form-urlencoded"},
        method="POST",
    )

    with urllib.request.urlopen(request, timeout=TIMEOUT) as response:
        return json.load(response)


def get_access_token(username: str) -> str:
    response = request_json(
        get_token_url(),
        {
            "grant_type": "client_credentials",
            "client_id": CLIENT_ID,
            "client_secret": CLIENT_SECRET,
            "username": username,
        },
    )

    token = response.get("access_token")
    if not isinstance(token, str) or not token:
        raise RuntimeError("the token endpoint response does not contain access_token")
    return token


def build_headers(raw_headers: list[str], token: str | None) -> dict[str, str]:
    headers: dict[str, str] = {}

    if token is not None:
        headers["Authorization"] = f"Bearer {token}"

    for raw_header in raw_headers:
        name, separator, value = raw_header.partition(":")
        if not separator or not name.strip():
            raise ValueError(f"invalid header: {raw_header!r}")
        headers[name.strip()] = value.strip()

    return headers


def call_api(request_args: RequestArguments, token: str | None) -> int:
    headers = build_headers(request_args.headers, token)
    body = "&".join(request_args.data).encode("utf-8") if request_args.data else None
    method = request_args.method or ("POST" if body is not None else "GET")

    request = urllib.request.Request(
        request_args.url,
        data=body,
        headers=headers,
        method=method,
    )

    try:
        with urllib.request.urlopen(request, timeout=TIMEOUT) as response:
            sys.stdout.buffer.write(response.read())
            if sys.stdout.isatty():
                print()
            return 0
    except urllib.error.HTTPError as error:
        sys.stderr.write(f"HTTP {error.code} {error.reason}\n")
        sys.stderr.buffer.write(error.read())
        if sys.stderr.isatty():
            print(file=sys.stderr)
        return 1


def main() -> int:
    args, request_args = parse_cli()

    try:
        token = get_access_token(args.user) if args.user else None
        return call_api(request_args, token)
    except (urllib.error.URLError, TimeoutError, ValueError, RuntimeError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
