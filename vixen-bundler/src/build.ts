/** biome-ignore-all lint/style/noNonNullAssertion: the values are always set */
import { rmSync } from 'node:fs';
import { join, normalize, parse, relative, resolve } from 'node:path';
import { parseArgs } from 'node:util';

// Invoked by `vixen_bundler::build` with the consumer crate as cwd.
const isDev = process.env.PROFILE !== 'release';

const { values } = parseArgs({
  args: Bun.argv.slice(2),
  options: {
    root: { type: 'string' },
    assetsPrefix: { type: 'string' },
    entryGlob: { type: 'string' },
    staticGlob: { type: 'string' },
    userConfig: { type: 'string' },
  },
  strict: true,
});

const cfg = values as Required<typeof values>;

const outdir = join(process.env.OUT_DIR!, cfg.assetsPrefix);

const entrypoints = Array.from(new Bun.Glob(cfg.entryGlob).scanSync()).sort();

const Reserved = ['root', 'outdir', 'metafile', 'naming'] as const;
type Reserved = (typeof Reserved)[number];

let userConfig: Omit<Bun.BuildConfig, Reserved> = { entrypoints: [] };
if (cfg.userConfig) {
  try {
    const namespace = await import(cfg.userConfig);
    const rawUserConfig = namespace.default;
    if (!rawUserConfig) {
      throw new Error(`user config ${cfg.userConfig} must have a default export`);
    }

    if (typeof rawUserConfig === 'function') {
      userConfig = await rawUserConfig();
    } else {
      userConfig = rawUserConfig;
    }

    if (typeof userConfig !== 'object') {
      throw new Error(
        `default export from ${cfg.userConfig} must be a Bun build config object or a function returning one`,
      );
    }
  } catch (error: any) {
    console.log(`cargo::error=vixen-bundler: Load user config error: ${error.message}`);
    process.exit(0);
  }
}

for (const field of Reserved) {
  if (Object.hasOwn(userConfig, field)) {
    console.log(
      `cargo::warning=vixen-bundler: user config field ${field} is managed by vixen-bundler and will be ignored`,
    );
  }
}

for (const entry of userConfig.entrypoints) {
  if (!entrypoints.includes(entry)) {
    entrypoints.push(entry);
  }
}

if (entrypoints.length === 0) {
  console.log(`cargo::error=vixen-bundler: nothing to bundle, no entry matched ${cfg.entryGlob}`);
  process.exit(0);
}

rmSync(outdir, { recursive: true, force: true });

const result = await Bun.build({
  splitting: true,
  minify: !isDev,
  sourcemap: isDev ? 'linked' : 'none',

  ...userConfig,

  entrypoints,
  root: cfg.root,
  outdir,
  metafile: true,
  naming: {
    entry: '[dir]/[name]-[hash].[ext]',
    chunk: 'chunk-[hash].[ext]',
    asset: '[dir]/[name]-[hash].[ext]',
  },
});

if (!result.success) {
  console.log('cargo::error=vixen-bundler: bun failed to bundle');
  process.exit(0);
}

const assetUrl = (path: string) => join('/', cfg.assetsPrefix, normalize(path)).replaceAll('\\', '/');

const entries: Record<string, { js: string; css: string | null }> = {};

const dir = resolve(outdir);
const files = result.outputs.map((b) => relative(dir, b.path)).sort();

for (const [rawPath, out] of Object.entries(result.metafile!.outputs)) {
  // CSS outputs share an entry point; `cssBundle` links them from the JS output.
  if (!out.entryPoint || !rawPath.endsWith('.js')) continue;

  const entry = {
    js: assetUrl(rawPath),
    css: out.cssBundle ? assetUrl(out.cssBundle) : null,
  };

  entries[resolve(normalize(out.entryPoint))] = entry;
}

// Copy `asset!` files into the bundle root as `[name]-[hash][ext]`.
const statics: Record<string, string> = {};
const copied = new Set<string>();

for (const path of Array.from(new Bun.Glob(cfg.staticGlob).scanSync()).sort()) {
  const bytes = await Bun.file(path).bytes();
  const hash = Bun.hash(bytes).toString(36).slice(-8).padStart(8, '0');
  const { name, ext } = parse(path);
  const out = `${name}-${hash}${ext}`;

  if (!copied.has(out)) {
    if (files.includes(out)) {
      console.log(`cargo::error=vixen-bundler: ${path} collides with bundle output ${out}`);
      process.exit(0);
    }
    await Bun.write(join(dir, out), bytes);
    copied.add(out);
    files.push(out);
  }

  statics[resolve(path)] = assetUrl(out);
  console.log(`cargo::rerun-if-changed=${resolve(path)}`);
}

const manifest = {
  entry_glob: cfg.entryGlob,
  static_glob: cfg.staticGlob,
  dir,
  prefix: cfg.assetsPrefix,
  entries,
  static: statics,
  files,
};

console.log(`cargo::rustc-env=VIXEN_MANIFEST=${JSON.stringify(manifest)}`);
