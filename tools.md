# Tools

1. Subscribe:
   
    `mosquitto_sub -t 'a/b/c' -t 'z/x/c' -t 'q/w/e'`

2. Subscribe and unsubscribe, keepalive 5:
    
    `mosquitto_sub -t 'a/b/c' -t 'z/x/c' -t 'q/w/e' -k 5 -U 'a/b/c' -U 'z/x/c' -U 'q/w/e' -d`

3. Publish:

    `mosquitto_pub -t 'a/b/c' -m 'test body' -d`
