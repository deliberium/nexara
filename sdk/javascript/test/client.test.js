import assert from "node:assert/strict";
import { afterEach, test } from "node:test";

import { NexaraClient } from "../index.js";

afterEach(() => {
  delete globalThis.fetch;
});

test("listTools sends prompt and bearer auth", async () => {
  const calls = [];
  globalThis.fetch = async (url, init = {}) => {
    calls.push({ url: String(url), init });
    return ok({ status: "ok", tools: [] });
  };

  const client = new NexaraClient({
    baseUrl: "http://localhost:8787/",
    token: "secret"
  });
  await client.listTools({ prompt: "search docs" });

  assert.equal(calls.length, 1);
  assert.equal(calls[0].url, "http://localhost:8787/v1/nexara/tools?prompt=search+docs");
  assert.deepEqual(calls[0].init.headers, { authorization: "Bearer secret" });
});

test("callTool posts JSON request body", async () => {
  const calls = [];
  globalThis.fetch = async (url, init = {}) => {
    calls.push({ url, init });
    return ok({ status: "ok", result: { answer: 42 } });
  };

  const client = new NexaraClient({ baseUrl: "http://localhost:8787", token: "secret" });
  const result = await client.callTool("search", { q: "nexara" });

  assert.deepEqual(result, { status: "ok", result: { answer: 42 } });
  assert.equal(calls[0].url, "http://localhost:8787/v1/nexara/tools/call");
  assert.deepEqual(calls[0].init.headers, {
    "content-type": "application/json",
    authorization: "Bearer secret"
  });
  assert.deepEqual(JSON.parse(calls[0].init.body), {
    tool: "search",
    params: { q: "nexara" }
  });
});

test("admin helpers use expected routes", async () => {
  const calls = [];
  globalThis.fetch = async (url, init = {}) => {
    calls.push({ url, init });
    return ok({ status: "ok" });
  };

  const client = new NexaraClient({ baseUrl: "http://localhost:8787", token: "secret" });
  await client.listSkills();
  await client.installSkill("manifest = true", { enabled: true });
  await client.setSkillEnabled("example/skill", true);
  await client.setSkillEnabled("example/skill", false);
  await client.getSkillReadiness("example/skill");
  await client.listSkillSecrets("example/skill");
  await client.putSkillSecret("example/skill", "API_TOKEN", "value");
  await client.deleteSkillSecret("example/skill", "API_TOKEN");

  assert.deepEqual(
    calls.map((call) => [call.init.method || "GET", call.url]),
    [
      ["GET", "http://localhost:8787/admin/nexara/skills"],
      ["POST", "http://localhost:8787/admin/nexara/skills/install"],
      ["POST", "http://localhost:8787/admin/nexara/skills/example%2Fskill/enable"],
      ["POST", "http://localhost:8787/admin/nexara/skills/example%2Fskill/disable"],
      ["GET", "http://localhost:8787/admin/nexara/skills/example%2Fskill/readiness"],
      ["GET", "http://localhost:8787/admin/nexara/skills/example%2Fskill/secrets"],
      ["POST", "http://localhost:8787/admin/nexara/skills/example%2Fskill/secrets/API_TOKEN"],
      ["DELETE", "http://localhost:8787/admin/nexara/skills/example%2Fskill/secrets/API_TOKEN"]
    ]
  );
  assert.deepEqual(JSON.parse(calls[1].init.body), {
    manifest_toml: "manifest = true",
    enabled: true
  });
  assert.deepEqual(JSON.parse(calls[6].init.body), { value: "value" });
});

test("failed responses throw method-specific errors", async () => {
  globalThis.fetch = async () => ({
    ok: false,
    status: 500,
    async json() {
      return {};
    }
  });

  const client = new NexaraClient({ baseUrl: "http://localhost:8787" });

  await assert.rejects(() => client.listTools(), /Nexara listTools failed: 500/);
  await assert.rejects(() => client.callTool("search"), /Nexara callTool failed: 500/);
  await assert.rejects(() => client.listSkills(), /Nexara listSkills failed: 500/);
});

function ok(payload) {
  return {
    ok: true,
    status: 200,
    async json() {
      return payload;
    }
  };
}
