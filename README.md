# Eden Server

Server component for [Eden](https://github.com/celaus/eden).

[![Build Status](https://travis-ci.org/celaus/eden-server.svg?branch=master)](https://travis-ci.org/celaus/eden-server)

Eden is a system to collect IoT data via a RESTful web interface.



# Architecture

eden-server is part of a larger system to store sensor data in CrateDB for analysis. Consequently to get the most out of it, all parts are required:

- [eden](https://github.com/celaus/eden)
- [eden-server](https://github.com/celaus/eden-server)
- [CrateDB](https://crate.io)

This diagram illustrates roughly how these components work together:

```
 _________                              _______________                      _____________
| RPi w/  |  HTTP/REST PUT             | Server        | rust-cratedb       | Server (II) |
| Sensors |  =======================>  |               | ================>  |             |
| <eden>  |  (JWT auth, JSON Payload)  | <eden-server> | bulk insert        |             |
|_________|                            |               |                    | <CrateDB>   |
                                       |               |                    |_____________|
 _________                             |               |                
| RPi2 w/ |  =======================>  |               |
| Sensors |                            |_______________|
| <eden>  |
|_________|
```
> Several eden clients can send data to eden-server's REST interface (authenticated with JWT). eden-server will then collect these requests and insert the received data into CrateDB using [rust-cratedb](https://github.com/celaus/rust-cratedb).
