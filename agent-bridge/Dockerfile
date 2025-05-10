FROM alpine:latest AS unzipper
RUN apk add unzip wget curl
RUN mkdir /opt/kconnect
WORKDIR /opt/kconnect
RUN wget https://github.com/lensesio/stream-reactor/releases/download/6.2.0/kafka-connect-mqtt-6.2.0.zip -O mqtt.zip
RUN unzip mqtt

FROM confluentinc/cp-kafka-connect-base:7.6.0
WORKDIR /usr/share/confluent-hub-components
COPY  --from=unzipper /opt/kconnect/ ./

#ADD https://github.com/lensesio/stream-reactor/releases/download/6.2.0/kafka-connect-mqtt-6.2.0.zip ./
#ADD https://packages.confluent.io/maven/io/confluent/kafka-mqtt/6.2.0/kafka-mqtt-6.2.0.jar ./
#COPY /kafka-connect-mqtt-6.2.0 .
