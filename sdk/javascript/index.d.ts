export interface NexaraClientOptions {
  baseUrl?: string;
  token?: string;
}

export declare class NexaraClient {
  constructor(options?: NexaraClientOptions);
  listTools(options?: { prompt?: string }): Promise<unknown>;
  callTool(tool: string, params?: Record<string, unknown>): Promise<unknown>;
  listSkills(): Promise<unknown>;
  installSkill(manifestToml: string, options?: { enabled?: boolean }): Promise<unknown>;
  setSkillEnabled(skillId: string, enabled: boolean): Promise<unknown>;
  getSkillReadiness(skillId: string): Promise<unknown>;
  listSkillSecrets(skillId: string): Promise<unknown>;
  putSkillSecret(skillId: string, secretName: string, value: string): Promise<unknown>;
  deleteSkillSecret(skillId: string, secretName: string): Promise<unknown>;
}
