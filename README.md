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

In ICMP mode the tool works as a regular ping. No server is required, root
permissions are required for the client.

## Options

```
  -T, --timeout <TIMEOUT>        [default: 30]
  -I, --interval <INTERVAL>      [default: 1.0]
  -S, --frame-size <FRAME_SIZE>  frame size (TCP/UDP) [default: 1500]
  -W, --latency-warn <WARN>
  -O, --output <OUTPUT_KIND>     output kind [default: regular] [possible values: regular, syslog, chart, ndjson]
```

* when *--latency-warn* option is specified (in seconds), logs only frames with
latency equal or greater than the specified number

* when *--output syslog* option is specified, logs all messages to syslog. Useful to
run the tool in the background or as a system service

* when *--output chart* option is specified, outputs the result as a live chart in the
console

<img src="https://raw.githubusercontent.com/alttch/latencymon/master/chart.png"
/>

* when *--output ndjson* option is specified, outputs the result as ndjson in
the following line format:

```json
{"t":1703460047.4284112,"v":0.009972634}
```

where "t" field is the event timestamp and "v" is latency in seconds. In case
of errors, "v" is set to -1.
