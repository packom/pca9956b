# pca9956b

A RESTful HTTP microservice for controlling PCA9956B devices.

This is currently a work in progress - most of the API is implemented, but not all of it.

## Building

```
git clone https://github.com/packom/pca9956b
cd pca9956b
cargo build
```

## Running

pca9956b uses environment variables for configuration, as it's intended to be run within a container.  You must have an instance of the [i2cbus](https://github.com/packom/i2cbus) microservice providing access to the appropriate I2C bus - use the I2CBUS_IP and I2CBUS_PORT variables to point to that.

To run bound to localhost:8080 with INFO level logging:

```
env SERVER_IP=localhost \
env SERVER_PORT=8080 \
env RUST_LOG=INFO \
env I2CBUS_IP=localhost \
env I2CBUS_PORT=8081 \
cargo run
```

Use environment variable HTTPS (no value is necessary) to enable HTTPS support, e.g.:

```
env SERVER_IP=localhost \
env SERVER_PORT=8443 \
env HTTPS= \
env RUST_LOG=INFO \
env I2CBUS_IP=localhost \
env I2CBUS_PORT=8081 \
cargo run
```

pca9956b expects to find certificate and key files at the following paths (which are not currently configurable):

```
/ssl/key.pem
/ssl/cert.pem
```

To see other options run:

```
cargo run -- --help
```

## Controlling the PCA9956B

To see examples controlling a PCA9956B device see [here](https://github.com/packom/pca9956b-api/blob/master/notes/examples.txt).
