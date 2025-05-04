#/bin/sh
curl -s -X PUT -H 'Content-Type: application/json' --data @config/nodes-mqtt-connector-config.json http://localhost:8083/connectors/nodes/config
curl -s -X PUT -H 'Content-Type: application/json' --data @config/agents-mqtt-connector-config.json http://localhost:8083/connectors/agents/config
curl -s -X PUT -H 'Content-Type: application/json' --data @config/processes-mqtt-connector-config.json http://localhost:8083/connectors/processes/config
curl -s -X PUT -H 'Content-Type: application/json' --data @config/networks-mqtt-connector-config.json http://localhost:8083/connectors/connections/config
curl -s -X PUT -H 'Content-Type: application/json' --data @config/listening-mqtt-connector-config.json http://localhost:8083/connectors/listening/config
curl -s -X GET -H 'Content-Type: application/json' http://localhost:8083/connectors/
