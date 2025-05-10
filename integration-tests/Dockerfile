FROM alpine:latest AS nodeagentsim
RUN apk add --no-cache mosquitto-clients && \
    ln -s /usr/bin/mosquitto_pub /usr/local/bin/pub && \
    ln -s /usr/bin/mosquitto_sub /usr/local/bin/sub

RUN mkdir /tests
ADD ./tests/* /tests/

#VOLUME /tests

ENV TEST_MQTT_HOST=localhost TEST_MQTT_PORT=9001

# Handles CTRL+C in container (ie if subscribe to topic)
RUN apk add --no-cache tini
# Tini is now available at /sbin/tini
ENTRYPOINT ["/sbin/tini", "--"]

# Run your program under Tini
CMD ["sh", "/tests/dummy.sh"]
# or docker run your-image /your/program ...
