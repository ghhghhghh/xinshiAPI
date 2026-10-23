import type {
  ApiRequest,
  Collection,
  Environment,
  ParamField,
  Workspace,
} from './types';

const AT = '2026-10-22T08:00:00.000Z';

function param(
  id: string,
  name: string,
  type: ParamField['type'],
  value: string,
  description: string,
  options: string[] = [],
): ParamField {
  return { id, enabled: true, name, type, value, fileName: '', description, options };
}

function baseRequest(
  input: Pick<ApiRequest, 'id' | 'collectionId' | 'name' | 'method' | 'url' | 'sortOrder'> &
    Partial<ApiRequest>,
): ApiRequest {
  return {
    description: '',
    params: [],
    headers: [],
    body: { mode: 'none', raw: '', contentType: 'application/json', fields: [] },
    auth: { type: 'inherit', bearerToken: '', username: '', password: '', keyName: 'Authorization', keyValue: '', keyIn: 'header' },
    extractors: [],
    responseLabels: [],
    updatedAt: AT,
    ...input,
  };
}

function folder(
  id: string,
  name: string,
  parentId: string | null,
  description: string,
  sortOrder: number,
  headers: ParamField[] = [],
  auth: Collection['auth'] = { type: 'inherit', bearerToken: '', username: '', password: '', keyName: 'Authorization', keyValue: '', keyIn: 'header' },
): Collection {
  return {
    id,
    parentId,
    name,
    description,
    headers,
    auth,
    sortOrder,
    createdAt: AT,
    updatedAt: AT,
  };
}

const collections: Collection[] = [
  folder('col-hr', '人事管理', null, '顶层文件夹就是一个项目', 0, [], {
    type: 'apikey',
    bearerToken: '',
    username: '',
    password: '',
    keyName: 'snyl-token',
    keyValue: '{{token}}',
    keyIn: 'header',
  }),
  folder('col-hr-staff', '人员管理', 'col-hr', '员工资料相关接口', 0),
  folder('col-hr-org', '组织架构', 'col-hr', '部门与岗位', 1),
  folder('col-demo', '入门示例', null, '带参数的 GET、JSON POST 与鉴权示例', 1),
  folder('col-jsonplaceholder', 'JSONPlaceholder', null, '公开测试接口，无需鉴权', 2),
];

const requests: ApiRequest[] = [
  baseRequest({
    id: 'req-hr-list',
    collectionId: 'col-hr-staff',
    name: '员工列表',
    method: 'GET',
    url: '/api/staff/list',
    sortOrder: 0,
    description: '路径用相对地址，前面会自动拼当前环境的前置 URL。',
    params: [
      param('p-hr-keyword', 'keyword', 'string', '', '姓名或工号'),
      param('p-hr-page', 'page', 'number', '1', '页码'),
      param('p-hr-size', 'pageSize', 'number', '20', '每页条数'),
      param('p-hr-status', 'status', 'enum', 'active', '在职状态', ['active', 'leave', 'all']),
    ],
  }),
  baseRequest({
    id: 'req-hr-detail',
    collectionId: 'col-hr-staff',
    name: '员工详情',
    method: 'GET',
    url: '/api/staff/detail',
    sortOrder: 1,
    description: '演示必填的枚举参数。',
    params: [param('p-hr-id', 'id', 'string', '1001', '员工 ID')],
  }),
  baseRequest({
    id: 'req-hr-create',
    collectionId: 'col-hr-staff',
    name: '新建员工',
    method: 'POST',
    url: '/api/staff/create',
    sortOrder: 2,
    description: 'JSON 请求体，可以切到「字段」视图逐个编辑。',
    headers: [param('h-hr-ct', 'Content-Type', 'string', 'application/json', 'JSON 请求体')],
    body: {
      mode: 'json',
      raw: '{\n  "name": "张三",\n  "mobile": "13800000000",\n  "deptId": 12,\n  "roles": ["staff"]\n}',
      contentType: 'application/json',
      fields: [],
    },
  }),
  baseRequest({
    id: 'req-hr-dept',
    collectionId: 'col-hr-org',
    name: '部门列表',
    method: 'GET',
    url: '/api/dept/tree',
    sortOrder: 0,
    description: '同一个项目下的请求共用 Authorization 头。',
  }),
  baseRequest({
    id: 'req-demo-get',
    collectionId: 'col-demo',
    name: '查询用户列表',
    method: 'GET',
    url: '/get',
    sortOrder: 0,
    description: '演示枚举参数、布尔参数与自定义下拉选项。',
    params: [
      param('p-get-keyword', 'keyword', 'string', 'rust', '搜索关键词'),
      param('p-get-sort', 'sort', 'enum', 'asc', '排序方向', ['asc', 'desc']),
      param('p-get-role', 'role', 'enum', 'admin', '角色筛选', ['admin', 'editor', 'viewer']),
      param('p-get-page', 'page', 'number', '1', '页码'),
      param('p-get-verbose', 'verbose', 'boolean', 'false', '返回调试信息'),
    ],
    headers: [
      param('h-get-accept', 'Accept', 'string', 'application/json', '期望响应类型'),
      param('h-get-source', 'X-Demo-Source', 'string', '信使', '示例自定义头'),
    ],
  }),
  baseRequest({
    id: 'req-demo-post',
    collectionId: 'col-demo',
    name: '创建订单（JSON）',
    method: 'POST',
    url: '/post',
    sortOrder: 1,
    description: '发送 JSON 请求体，响应会回显请求内容。',
    headers: [
      param('h-post-content-type', 'Content-Type', 'string', 'application/json', 'JSON 请求体'),
    ],
    body: {
      mode: 'json',
      raw: '{\n  "orderNo": "A-1001",\n  "amount": 128.5,\n  "channel": "web",\n  "items": [\n    { "sku": "RUST-01", "qty": 2 }\n  ]\n}',
      contentType: 'application/json',
      fields: [],
    },
  }),
  baseRequest({
    id: 'req-demo-form',
    collectionId: 'col-demo',
    name: '表单提交（form-data）',
    method: 'POST',
    url: '/post',
    sortOrder: 2,
    description: '演示 multipart/form-data 的字段表格。',
    body: {
      mode: 'form-data',
      raw: '',
      contentType: '',
      fields: [
        param('f-demo-name', 'username', 'string', 'admin', '用户名'),
        param('f-demo-pass', 'password', 'string', 'admin1', '密码'),
      ],
    },
  }),
  baseRequest({
    id: 'req-demo-auth',
    collectionId: 'col-demo',
    name: 'Bearer 鉴权示例',
    method: 'GET',
    url: '/bearer',
    sortOrder: 3,
    description: 'Token 来自当前环境的 {{token}} 变量。',
    auth: {
      type: 'bearer',
      bearerToken: '{{token}}',
      username: '',
      password: '',
      keyName: 'Authorization',
      keyValue: '',
      keyIn: 'header',
    },
  }),
  baseRequest({
    id: 'req-demo-404',
    collectionId: 'col-demo',
    name: '错误状态示例',
    method: 'GET',
    url: '/status/404',
    sortOrder: 4,
    description: '用来看 4xx 响应的展示效果。',
  }),
  baseRequest({
    id: 'req-jp-user',
    collectionId: 'col-jsonplaceholder',
    name: '获取用户 1',
    method: 'GET',
    url: 'https://jsonplaceholder.typicode.com/users/1',
    sortOrder: 0,
    description: '一个完整的公开接口，写全协议时不会被前置 URL 覆盖。',
  }),
  baseRequest({
    id: 'req-jp-create',
    collectionId: 'col-jsonplaceholder',
    name: '创建文章',
    method: 'POST',
    url: 'https://jsonplaceholder.typicode.com/posts',
    sortOrder: 1,
    description: '演示 JSON 请求体的发送。',
    headers: [
      param('h-jp-content-type', 'Content-Type', 'string', 'application/json', 'JSON 请求体'),
    ],
    body: {
      mode: 'json',
      raw: '{\n  "title": "信使测试",\n  "body": "hello from messenger",\n  "userId": 1\n}',
      contentType: 'application/json',
      fields: [],
    },
  }),
];

const environments: Environment[] = [
  {
    id: 'env-demo',
    name: '开发环境',
    baseUrl: 'https://httpbin.org',
    createdAt: AT,
    updatedAt: AT,
    variables: [
      {
        id: 'v-token',
        enabled: true,
        name: 'token',
        value: 'demo-token-123',
        description: '演示用 Token',
      },
      {
        id: 'v-page',
        enabled: true,
        name: 'pageSize',
        value: '10',
        description: '分页大小',
      },
    ],
  },
  {
    id: 'env-local',
    name: '本地开发',
    baseUrl: 'http://localhost:3000',
    createdAt: AT,
    updatedAt: AT,
    variables: [],
  },
  {
    id: 'env-test',
    name: '测试环境',
    baseUrl: 'http://testapp.sny156.com',
    createdAt: AT,
    updatedAt: AT,
    variables: [],
  },
];

export function createSeedWorkspace(): Workspace {
  return {
    collections: collections.map((item) => ({
      ...item,
      headers: item.headers.map((field) => ({ ...field, options: [...field.options] })),
    })),
    requests: requests.map((item) => ({
      ...item,
      params: item.params.map((field) => ({ ...field, options: [...field.options] })),
      headers: item.headers.map((field) => ({ ...field, options: [...field.options] })),
      body: {
        ...item.body,
        fields: item.body.fields.map((field) => ({ ...field, options: [...field.options] })),
      },
      auth: { ...item.auth },
    })),
    environments: environments.map((item) => ({
      ...item,
      variables: item.variables.map((variable) => ({ ...variable })),
    })),
    activeEnvironmentId: 'env-demo',
  };
}
