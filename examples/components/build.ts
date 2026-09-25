import tailwind from 'bun-plugin-tailwind';

export default (): Omit<Bun.BuildConfig, 'root' | 'outdir' | 'metafile' | 'naming'> => ({
  entrypoints: [],
  plugins: [tailwind],
});
