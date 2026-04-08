interface ImportMetaEnv {
  readonly BASE_URL?: string;
  readonly DEV: boolean;
  readonly MODE?: string;
  readonly PROD: boolean;
  readonly [key: string]: boolean | string | undefined;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
  readonly url: string;
}

declare module '*.css' {
  const cssUrl: string;
  export default cssUrl;
}
