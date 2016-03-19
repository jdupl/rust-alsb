# Blueprint

Random data is XORed to payload

### Random data
* 4 bytes: random length of random data
* n bytes: random data
* Note: Both length and data must be unique each time a file is encoded

### Payload
* 4 bytes: length of payload
* n bytes: payload data
