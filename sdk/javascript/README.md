# Nexara JavaScript SDK

Tiny HTTP client for the Nexara server list/call API.

```js
import { NexaraClient } from "@nexara/runtime";

const client = new NexaraClient({ baseUrl: "http://127.0.0.1:8787", token: "dev" });
const tools = await client.listTools({ prompt: "check status" });
```
