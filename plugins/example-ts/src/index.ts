import { Bridge, Streams, UpdateOp } from 'iii-sdk';

const ENGINE_URL = 'ws://127.0.0.1:49134';
const PLUGIN_NAME = 'example-ts';
const PLUGIN_VERSION = '0.1.0';

const bridge = new Bridge(ENGINE_URL, {
  runtime: 'node',
  version: PLUGIN_VERSION,
  name: `plugin.${PLUGIN_NAME}`,
  os: process.platform,
});

await bridge.connect();
console.log(`[${PLUGIN_NAME}] Connected to iii engine at ${ENGINE_URL}`);

const streams = new Streams(bridge);

bridge.registerFunction(
  'plugin.example-ts.greet',
  async (input: Record<string, unknown>) => {
    const name =
      typeof input.name === 'string' ? input.name : 'world';
    const style =
      typeof input.style === 'string' ? input.style : 'friendly';

    console.log(
      `[${PLUGIN_NAME}] greet called — name=${name}, style=${style}`
    );

    const greetings: Record<string, string> = {
      friendly: `Hey ${name}! Welcome from the TypeScript example plugin.`,
      formal: `Good day, ${name}. This is the TypeScript example plugin speaking.`,
      pirate: `Ahoy ${name}! Ye be talkin' to the TypeScript plugin, arrr!`,
    };

    const message = greetings[style] ?? greetings.friendly;

    return {
      message,
      plugin: PLUGIN_NAME,
      version: PLUGIN_VERSION,
      language: 'typescript',
    };
  }
);
console.log(`[${PLUGIN_NAME}] Registered: plugin.example-ts.greet`);

bridge.registerFunction(
  'plugin.example-ts.stats',
  async (input: Record<string, unknown>) => {
    const scope =
      typeof input.scope === 'string' ? input.scope : 'agents';
    const key = typeof input.key === 'string' ? input.key : undefined;

    console.log(
      `[${PLUGIN_NAME}] stats called — scope=${scope}, key=${key ?? '(all)'}`
    );

    if (key) {
      const result = await streams.update(`${scope}::${key}`, []);
      return {
        scope,
        key,
        value: result.newValue,
        exists: result.newValue !== null && result.newValue !== undefined,
      };
    }

    const stateKey = `plugin_state::${PLUGIN_NAME}::stats_calls`;
    await streams.update(stateKey, [
      UpdateOp.merge({ invocations: 1, lastScope: scope }),
    ]);

    return {
      scope,
      note: 'Provide a "key" to fetch a specific entry from the scope.',
      plugin: PLUGIN_NAME,
    };
  }
);
console.log(`[${PLUGIN_NAME}] Registered: plugin.example-ts.stats`);

bridge.registerFunction(
  'plugin.example-ts.echo-stream',
  async (input: Record<string, unknown>) => {
    const channel =
      typeof input.channel === 'string'
        ? input.channel
        : 'plugin-echo';
    const data = input.data ?? { ts: Date.now() };

    console.log(`[${PLUGIN_NAME}] echo-stream called — channel=${channel}`);

    const stateKey = `streams::${channel}`;
    const result = await streams.update(stateKey, [
      UpdateOp.merge({
        lastMessage: data,
        updatedAt: new Date().toISOString(),
        source: PLUGIN_NAME,
      }),
    ]);

    return {
      channel,
      written: true,
      currentState: result.newValue,
    };
  }
);
console.log(`[${PLUGIN_NAME}] Registered: plugin.example-ts.echo-stream`);

bridge.registerFunction(
  'plugin.example-ts.manifest',
  async () => {
    return {
      id: `plugin.${PLUGIN_NAME}`,
      name: 'Example TypeScript Plugin',
      version: PLUGIN_VERSION,
      language: 'typescript',
      description:
        'A reference TypeScript plugin for rimuru that demonstrates function registration, state reads, and stream writes.',
      functions: [
        {
          id: 'plugin.example-ts.greet',
          description: 'Returns a greeting message with configurable style',
          inputSchema: {
            type: 'object',
            properties: {
              name: { type: 'string', description: 'Name to greet' },
              style: {
                type: 'string',
                enum: ['friendly', 'formal', 'pirate'],
                default: 'friendly',
              },
            },
          },
        },
        {
          id: 'plugin.example-ts.stats',
          description:
            'Reads a value from iii state by scope and key',
          inputSchema: {
            type: 'object',
            properties: {
              scope: {
                type: 'string',
                description: 'State scope to query (e.g. agents, sessions, costs)',
                default: 'agents',
              },
              key: {
                type: 'string',
                description: 'Specific key within the scope',
              },
            },
          },
        },
        {
          id: 'plugin.example-ts.echo-stream',
          description:
            'Writes arbitrary data to a named stream channel via iii Streams',
          inputSchema: {
            type: 'object',
            properties: {
              channel: {
                type: 'string',
                description: 'Stream channel name',
                default: 'plugin-echo',
              },
              data: {
                description: 'Arbitrary JSON data to write',
              },
            },
          },
        },
      ],
      hooks: [],
    };
  }
);
console.log(`[${PLUGIN_NAME}] Registered: plugin.example-ts.manifest`);

console.log(
  `[${PLUGIN_NAME}] Plugin ready — press Ctrl+C to stop`
);

process.on('SIGINT', () => {
  console.log(`[${PLUGIN_NAME}] Shutting down`);
  bridge.disconnect();
  process.exit(0);
});

process.on('SIGTERM', () => {
  console.log(`[${PLUGIN_NAME}] Received SIGTERM, shutting down`);
  bridge.disconnect();
  process.exit(0);
});
