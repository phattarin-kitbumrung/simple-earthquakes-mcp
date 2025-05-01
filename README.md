# Simple Earthquakes MCP

## Overview
This project collect earthquake data in Thailand using rust mcp server. 
It shows recent earthquakes and provides details about each event.
After that client in python will connect with ollama local llm to process last result.

## Installation
1. Clone this repository
```
git clone https://github.com/phattarin-kitbumrung/simple-earthquakes-mcp.git
```
2. Start the rust mcp server (earthquakes-mcp-rust)
```
cargo run
```
3. Run the client in python (earthquakes-python)
```
python3 client.py
```

## Acknowledgements
- Data provided by https://theactive.thaipbs.or.th/data/earthquake-in-thailand
