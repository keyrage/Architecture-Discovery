#/bin/sh
curl -s -X DELETE -H 'Content-Type: application/json' http://localhost:8083/connectors/nodes
curl -s -X DELETE -H 'Content-Type: application/json' http://localhost:8083/connectors/agents
curl -s -X DELETE -H 'Content-Type: application/json' http://localhost:8083/connectors/processes
curl -s -X DELETE -H 'Content-Type: application/json' http://localhost:8083/connectors/net-connections
curl -s -X DELETE -H 'Content-Type: application/json' http://localhost:8083/connectors/net-listening
curl -s -X GET -H 'Content-Type: application/json' http://localhost:8083/connectors/
