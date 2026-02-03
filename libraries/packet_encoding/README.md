# SLAM Robot

This repo houses the software for a skid-steer robot that does SLAM.
The basic architecture is:

```
Cameras -> Onboard PC -> Embedded devices
               |
               V
          Web Server for control
```

The communication of the cameras is USB, the communication to the embedded devices and webpage is via a COBS (Consistent Overhead Byte Stuffing), CBOR (Concise Binary Object Representation) encoding. Communication is a pub-sub architecture.

The frontend (`web_interface`) is written in typescript, but uses a WASM bundle to handle packet encoding/decoding. Communication is over websocket.


