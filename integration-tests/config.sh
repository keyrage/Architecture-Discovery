#/bin/sh
curl -s -X PUT -H 'Content-Type: application/json' --data @nodes-mqtt-connector-config.json http://localhost:8083/connectors/nodes/config
curl -s -X PUT -H 'Content-Type: application/json' --data @agents-mqtt-connector-config.json http://localhost:8083/connectors/agents/config
curl -s -X PUT -H 'Content-Type: application/json' --data @processes-mqtt-connector-config.json http://localhost:8083/connectors/processes/config
curl -s -X PUT -H 'Content-Type: application/json' --data @networks-mqtt-connector-config.json http://localhost:8083/connectors/connections/config
curl -s -X PUT -H 'Content-Type: application/json' --data @listening-mqtt-connector-config.json http://localhost:8083/connectors/listening/config
curl -s -X GET -H 'Content-Type: application/json' http://localhost:8083/connectors/
