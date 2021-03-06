# RatelMQ

![CI](https://github.com/ratelmq/ratelmq/workflows/CI/badge.svg)
![DockerHub](https://img.shields.io/docker/pulls/ratelmq/ratelmq)
![GitHub](https://img.shields.io/github/license/ratelmq/ratelmq)

**RatelMQ is an efficient, reliable & scalable MQTT broker.**

---

## Installation

### Docker image

RatelMQ images are available on [Docker Hub](https://hub.docker.com/r/ratelmq/ratelmq).

Start container: `docker run --name ratelmq -p 1883:1883 ratelmq/ratelmq:main`

### Precompiled binaries

TBD

### Building from sources

TBD

## Configuration

The main configuration file is located in the `/etc/ratelmq/ratelmq.conf`.
RatelMQ supports configuration via a [TOML](https://github.com/toml-lang/toml) file and environment variables.

Environment variables take precedence and overwrites those from the configuration file.
When using environment variables:

* add `RATELMQ_` prefix
* separate nested items with `__`, e.g. `RATELMQ_MQTT__SOMETHING`

For the default values with description please see [config/ratelmq.toml](config/ratelmq.toml).

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for details.

## Roadmap

See [ROADMAP.md](ROADMAP.md) for details.

## Versioning

TBD

## Resources

* MQTT 3.1.1 spec: <https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html>

## Contributing

TBD

## License

The project is licensed under the Apache License 2.0, see [LICENSE](LICENSE).
