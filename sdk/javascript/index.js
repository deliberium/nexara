export class NexaraClient {
  constructor({ baseUrl, token } = {}) {
    this.baseUrl = (baseUrl || "http://127.0.0.1:8787").replace(/\/$/, "");
    this.token = token;
  }

  headers() {
    return this.token ? { authorization: `Bearer ${this.token}` } : {};
  }

  async listTools({ prompt } = {}) {
    const url = new URL(`${this.baseUrl}/v1/nexara/tools`);
    if (prompt) url.searchParams.set("prompt", prompt);
    const response = await fetch(url, { headers: this.headers() });
    if (!response.ok) throw new Error(`Nexara listTools failed: ${response.status}`);
    return response.json();
  }

  async callTool(tool, params = {}) {
    const response = await fetch(`${this.baseUrl}/v1/nexara/tools/call`, {
      method: "POST",
      headers: { "content-type": "application/json", ...this.headers() },
      body: JSON.stringify({ tool, params })
    });
    if (!response.ok) throw new Error(`Nexara callTool failed: ${response.status}`);
    return response.json();
  }

  async listSkills() {
    const response = await fetch(`${this.baseUrl}/admin/nexara/skills`, {
      headers: this.headers()
    });
    if (!response.ok) throw new Error(`Nexara listSkills failed: ${response.status}`);
    return response.json();
  }

  async installSkill(manifestToml, { enabled = false } = {}) {
    const response = await fetch(`${this.baseUrl}/admin/nexara/skills/install`, {
      method: "POST",
      headers: { "content-type": "application/json", ...this.headers() },
      body: JSON.stringify({ manifest_toml: manifestToml, enabled })
    });
    if (!response.ok) throw new Error(`Nexara installSkill failed: ${response.status}`);
    return response.json();
  }

  async setSkillEnabled(skillId, enabled) {
    const action = enabled ? "enable" : "disable";
    const response = await fetch(`${this.baseUrl}/admin/nexara/skills/${encodeURIComponent(skillId)}/${action}`, {
      method: "POST",
      headers: this.headers()
    });
    if (!response.ok) throw new Error(`Nexara setSkillEnabled failed: ${response.status}`);
    return response.json();
  }

  async getSkillReadiness(skillId) {
    const response = await fetch(`${this.baseUrl}/admin/nexara/skills/${encodeURIComponent(skillId)}/readiness`, {
      headers: this.headers()
    });
    if (!response.ok) throw new Error(`Nexara getSkillReadiness failed: ${response.status}`);
    return response.json();
  }

  async listSkillSecrets(skillId) {
    const response = await fetch(`${this.baseUrl}/admin/nexara/skills/${encodeURIComponent(skillId)}/secrets`, {
      headers: this.headers()
    });
    if (!response.ok) throw new Error(`Nexara listSkillSecrets failed: ${response.status}`);
    return response.json();
  }

  async putSkillSecret(skillId, secretName, value) {
    const response = await fetch(`${this.baseUrl}/admin/nexara/skills/${encodeURIComponent(skillId)}/secrets/${encodeURIComponent(secretName)}`, {
      method: "POST",
      headers: { "content-type": "application/json", ...this.headers() },
      body: JSON.stringify({ value })
    });
    if (!response.ok) throw new Error(`Nexara putSkillSecret failed: ${response.status}`);
    return response.json();
  }

  async deleteSkillSecret(skillId, secretName) {
    const response = await fetch(`${this.baseUrl}/admin/nexara/skills/${encodeURIComponent(skillId)}/secrets/${encodeURIComponent(secretName)}`, {
      method: "DELETE",
      headers: this.headers()
    });
    if (!response.ok) throw new Error(`Nexara deleteSkillSecret failed: ${response.status}`);
    return response.json();
  }
}
