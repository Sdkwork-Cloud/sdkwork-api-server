declare module 'node:fs' {
  export function existsSync(path: string): boolean;
}

declare module 'node:path' {
  const path: {
    dirname(filePath: string): string;
    join(...paths: string[]): string;
    resolve(...paths: string[]): string;
  };

  export default path;
}

declare module 'node:url' {
  export function fileURLToPath(url: string | URL): string;
}

declare const process: {
  readonly env: Record<string, string | undefined>;
};
