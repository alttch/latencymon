# latencymon

TCP/UDP/ICMP latency monitoring tool. Detects network anomalies.

## Usage example

### TCP/UDP

```
latencymon client tcp 10.90.1.7:9999
```

TCP/UDP mode requires a server running on the remote:

```
latencymon server tcp 0.0.0.0:9999
```

### ICMP (ping)

```
latencymon client icmp 10.90.1.7
```

In ICMP mode the tool works as the regular ping program. No server is required,
root permissions are required for the client.

## Options

```
  -T, --timeout <TIMEOUT>        [default: 30]
  -I, --interval <INTERVAL>      [default: 1.0]
  -S, --frame-size <FRAME_SIZE>  frame size (TCP/UDP) [default: 1500]
  -W, --latency-warn <WARN>
      --syslog                   log to syslog
  -C, --chart                    output result as a live chart
```

* when *--latency-warn* option is specified, logs only frames with latency
greater than the specified number

* when *--syslog* option is specified, logs all messages to syslog. Useful to
run the tool in background or as a system service

* when *--chart* option is specified, outputs the result as a live chart in the
console

<img src="https://raw.githubusercontent.com/alttch/latencymon/master/chart.png"
/>

