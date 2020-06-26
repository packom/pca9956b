FROM piersfinlayson/build-amd64:0.3.6 as pca9956b
WORKDIR /home/build/builds
RUN git clone https://github.com/packom/pca9956b
WORKDIR /home/build/builds/pca9956b
ARG TYPE
ARG TARGET
RUN cargo build $TYPE --target=$TARGET

FROM scratch
WORKDIR /
ARG TARGET
ARG COPY_TYPE
COPY --from=pca9956b /home/build/builds/pca9956b/target/$TARGET/$COPY_TYPE/pca9956b ./pca9956b
ARG DIR
ADD $DIR/api.yaml /static/
VOLUME ["/ssl"]
EXPOSE 8080
CMD ["/pca9956b"]
