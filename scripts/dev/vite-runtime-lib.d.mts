export function findReadableModuleResolution(options: {
  appRoot: string;
  donorRoots?: string[];
  specifier: string;
}): {
  candidateRoot: string;
  resolvedPath: string;
};

export function importReadablePackageDefault<T = unknown>(options: {
  appRoot: string;
  donorRoots?: string[];
  packageName: string;
  relativeEntry: string | string[];
}): Promise<T>;

export function resolveReadablePackageEntry(options: {
  appRoot: string;
  donorRoots?: string[];
  packageName: string;
  relativeEntry: string | string[];
}): string;

export function resolveReadablePackageRoot(options: {
  appRoot: string;
  donorRoots?: string[];
  packageName: string;
}): string;

export function resolveWorkspaceDonorRoots(appRoot: string): string[];
