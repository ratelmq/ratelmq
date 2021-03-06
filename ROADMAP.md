# Roadmap

This file aims to present a direction of the RatelMQ development.
This is a list of features that are meant to be implemented.
Please feel free to create a PR if You think that we are missing something!

*Because of the early stage of development, the features and dates below are subject to change.*

## 0.1 - 01.04.2021

1. Basic MQTT 3.1 & 3.1.1 support
   1. QoS 0 only
   2. No wildcard subscriptions
2. Multiple listeners
3. Configuration through file and ENVs
4. Docker image

## 0.2 - TBD

1. Wildcard subscriptions
2. Auth - user & password
3. Websockets

## 0.3 - TBD

1. Full MQTT 3.1.1 support 
   1. QoS
   2. Last will
   3. Retained messages
2. Shared subscriptions
3. SSL/TLS support

## 0.4 - TBD

1. Config validator
2. MQTT 5
3. DEB & RPM packages and repos?
4. Performance tests
5. Fuzz tests

## 0.5 - TBD

1. Monitoring
    1. SYS topics
    2. Prometheus/Graphite exporter
    3. Sample Grafana dashboard
2. Auth
    1. OAuth/JWT based
    2. Webhook
4. Extensibility
    1. HTTP API
    2. Webhooks

## 0.6 - TBD

1. HTTP dashboard
2. Management CLI app
3. Plugins
4. Proxy protocol

## 0.7 - TBD

1. Clustering

## 1.0 - TBD

1. Stabilization of broker, APIs
2. Performance
3. Resources usage model estimation
3. Deployment examples
