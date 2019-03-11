## Installing protoc on linux
Simply download the [protoc release](https://github.com/protocolbuffers/protobuf/releases) for your platform.

### Example: Downloading and installing on Linux
```
PROTOC_VERSION=3.7.0
PROTOC_ZIP=protoc-$PROTOC_VERSION-linux-x86_64.zip
curl -OL https://github.com/google/protobuf/releases/download/v$PROTOC_VERSION/$PROTOC_ZIP
sudo unzip -o $PROTOC_ZIP -d /usr/local bin/protoc
sudo chmod 755 /usr/local/bin/protoc
rm -f $PROTOC_ZIP
```

## Issues
- https://github.com/stepancheg/rust-protobuf/issues/397