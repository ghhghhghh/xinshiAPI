import { createEnvironment, type Workspace } from './types';

/**
 * 首次启动的空工作台：只给一个默认环境，
 * 不带任何示例项目、请求或历史，装好就是干净的。
 */
export function createSeedWorkspace(): Workspace {
  const environment = createEnvironment('本地开发', '');
  return {
    collections: [],
    requests: [],
    environments: [environment],
    activeEnvironmentId: environment.id,
  };
}
