#!/usr/bin/env python3
"""Serve signed CI config and proxy it to the real local voting services."""

import argparse
import base64
import http.client
import http.server
import json
import ssl
import subprocess
from pathlib import Path
from urllib.parse import urlsplit


class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, directory: str, **kwargs):
        self.root = Path(directory)
        super().__init__(*args, directory=directory, **kwargs)

    def do_GET(self):
        parts = urlsplit(self.path)
        if parts.path.startswith("/vote/"):
            try:
                rounds = grpc_rounds()
                route = parts.path[len("/vote"):]
                if route == "/shielded-vote/v1/rounds":
                    self.json({"rounds": rounds})
                elif route == "/shielded-vote/v1/rounds/overview":
                    self.json({"current_rounds": rounds})
                elif route.startswith("/shielded-vote/v1/round/"):
                    round_id = route.rsplit("/", 1)[-1].lower()
                    round_data = next((r for r in rounds if round_id_of(r) == round_id), None)
                    if round_data is None:
                        self.json({"error": "round not found"}, status=404)
                    else:
                        self.json({"round": round_data})
                else:
                    self.json({"error": "unsupported vote route"}, status=404)
            except subprocess.CalledProcessError as error:
                self.json({"error": error.stderr}, status=502)
            return
        for prefix, port in (("/pir/", 3000),):
            if parts.path.startswith(prefix):
                target = parts.path[len(prefix) - 1 :]
                if parts.query:
                    target += "?" + parts.query
                connection = http.client.HTTPConnection("127.0.0.1", port, timeout=30)
                try:
                    connection.request("GET", target)
                    response = connection.getresponse()
                    body = response.read()
                    self.send_response(response.status)
                    content_type = response.getheader("Content-Type")
                    if content_type:
                        self.send_header("Content-Type", content_type)
                    self.send_header("Content-Length", str(len(body)))
                    self.end_headers()
                    self.wfile.write(body)
                finally:
                    connection.close()
                return
        super().do_GET()

    def json(self, body, status=200):
        data = json.dumps(body).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)


def round_id_of(round_data):
    return base64.b64decode(round_data["vote_round_id"]).hex()


def json_u64(value):
    """grpcurl renders protobuf uint64 fields as strings; REST uses numbers."""
    return int(value) if value is not None else None


def grpc_rounds():
    result = subprocess.run(
        [args.grpcurl, "-plaintext", "-import-path", args.proto_root,
         "-proto", "svote/v1/query.proto", "-d", "{}", args.grpc,
         "svote.v1.Query/ListRounds"],
        check=True, capture_output=True, text=True,
    )
    payload = json.loads(result.stdout)
    status = {
        "SESSION_STATUS_ACTIVE": 1,
        "SESSION_STATUS_TALLYING": 2,
        "SESSION_STATUS_FINALIZED": 3,
        "SESSION_STATUS_PENDING": 4,
        "SESSION_STATUS_CEREMONY_FAILED": 5,
    }
    return [{
        "vote_round_id": round_data["voteRoundId"],
        "snapshot_height": json_u64(round_data.get("snapshotHeight")),
        "nullifier_imt_root": round_data.get("nullifierImtRoot"),
        "nc_root": round_data.get("ncRoot"),
        "ea_pk": round_data.get("eaPk"),
        "title": round_data.get("title"),
        "description": round_data.get("description"),
        "status": status.get(round_data.get("status"), round_data.get("status")),
        "vote_end_time": json_u64(round_data.get("voteEndTime")),
        "proposals": round_data.get("proposals", []),
    } for round_data in payload.get("rounds", [])]


parser = argparse.ArgumentParser()
parser.add_argument("--directory", required=True)
parser.add_argument("--cert", required=True)
parser.add_argument("--key", required=True)
parser.add_argument("--port", type=int, default=8443)
parser.add_argument("--grpcurl", required=True)
parser.add_argument("--proto-root", required=True)
parser.add_argument("--grpc", default="127.0.0.1:9190")
args = parser.parse_args()

server = http.server.ThreadingHTTPServer(("127.0.0.1", args.port),
                                         lambda *a, **kw: Handler(*a, directory=args.directory, **kw))
context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
context.load_cert_chain(args.cert, args.key)
server.socket = context.wrap_socket(server.socket, server_side=True)
server.serve_forever()
