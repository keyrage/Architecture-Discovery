FROM alpine:latest
RUN apk add curl
RUN mkdir /config
ADD ./*.sh ./*.json /config/

ENV TEST_BRIDGE_HOST=localhost

# Handles CTRL+C in container (ie if subscribe to topic)
RUN apk add --no-cache tini
# Tini is now available at /sbin/tini
ENTRYPOINT ["/sbin/tini", "--"]

# Run your program under Tini
CMD ["sh", "/config/config.sh"]
# or docker run your-image /your/program ...