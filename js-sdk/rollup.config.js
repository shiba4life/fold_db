import typescript from '@rollup/plugin-typescript';
import resolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';

export default [
  // ES Module build
  {
    input: 'src/index.ts',
    output: {
      file: 'dist/index.esm.js',
      format: 'es',
      sourcemap: true
    },
    plugins: [
      resolve({ browser: true }),
      commonjs(),
      typescript({
        outDir: 'dist',
        declaration: true,
        declarationMap: true
      })
    ],
    external: ['@noble/ed25519']
  },
  // CommonJS build
  {
    input: 'src/index.ts',
    output: {
      file: 'dist/index.js',
      format: 'cjs',
      sourcemap: true
    },
    plugins: [
      resolve({ browser: true }),
      commonjs(),
      typescript({
        outDir: 'dist',
        declaration: false
      })
    ],
    external: ['@noble/ed25519']
  }
];