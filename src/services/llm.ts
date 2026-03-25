/**
 * LLM 服务模块
 * 提供基于 OpenAI 格式的大语言模型调用功能
 * 使用单例模式维护全局配置
 */

// ==================== 类型定义 ====================

/**
 * 消息角色类型
 */
export type MessageRole = "system" | "user" | "assistant";

/**
 * 聊天消息
 */
export interface ChatMessage {
  /** 消息角色 */
  role: MessageRole;
  /** 消息内容 */
  content: string;
}

/**
 * LLM 配置选项
 */
export interface LLMConfig {
  /** API 基础 URL（如 https://api.openai.com） */
  baseUrl: string;
  /** API 密钥 */
  apiKey: string;
  /** 模型名称（如 gpt-3.5-turbo, gpt-4 等） */
  model: string;
  /** 系统提示词（默认助手角色） */
  systemPrompt?: string;
  /** 温度参数（0-2，控制随机性，默认 1） */
  temperature?: number;
  /** 最大生成 token 数（默认无限制） */
  maxTokens?: number;
  /** 停止序列（可选） */
  stop?: string[];
  /** top_p 采样参数（0-1，默认 1） */
  topP?: number;
  /** 频率惩罚（-2.0 到 2.0，默认 0） */
  frequencyPenalty?: number;
  /** 存在惩罚（-2.0 到 2.0，默认 0） */
  presencePenalty?: number;
  /** 是否启用 */
  enabled: boolean;
}

/**
 * LLM 响应结果
 */
export interface LLMResponse {
  /** 生成的消息列表 */
  choices: Array<{
    index: number;
    message: {
      role: MessageRole;
      content: string;
    };
    finish_reason: string;
  }>;
  /** 使用的模型 */
  model: string;
  /** 提示词使用的 token 数 */
  prompt_tokens: number;
  /** 完成使用的 token 数 */
  completion_tokens: number;
  /** 总 token 数 */
  total_tokens: number;
}

/**
 * 流式调用回调
 */
export type StreamCallback = (chunk: string) => void;

// ==================== LLM Service 类 ====================

/**
 * LLM 服务类（单例）
 * 维护全局配置，提供简化的调用接口
 */
export class LLMService {
  private static instance: LLMService;

  // 配置属性（可直接访问和修改）
  public config: LLMConfig;

  // 私有构造函数（防止外部直接实例化）
  private constructor() {
    this.config = this.get_config_from_local_storage();
  }

  private get_config_from_local_storage(): LLMConfig {
    const config = localStorage.getItem("llm_config");
    if (config) {
      return JSON.parse(config);
    }
    return {
      baseUrl: "",
      apiKey: "",
      model: "",
      systemPrompt: "",
      temperature: 1,
      topP: 1,
      frequencyPenalty: 0,
      presencePenalty: 0,
      enabled: false,
    }
  }

  private save_config_to_local_storage(config: LLMConfig) {
    localStorage.setItem("llm_config", JSON.stringify(config));
  }

  public updateConfig(config: Partial<LLMConfig>): void {
    this.config = { ...this.config, ...config };
    this.save_config_to_local_storage(this.config);
  }

  /**
   * 获取全局单例实例
   * @returns LLMService 实例
   */
  public static getInstance(): LLMService {
    if (!LLMService.instance) {
      LLMService.instance = new LLMService();
    }
    return LLMService.instance;
  }

  /**
   * 初始化配置
   * @param config 配置对象
   */
  public initialize(config: Partial<LLMConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * 检查配置是否有效
   * @returns 是否配置有效
   */
  public isConfigured(): boolean {
    return !!this.config.baseUrl && !!this.config.apiKey && !!this.config.model;
  }

  /**
   * 基础调用方法
   * @param messages 消息列表
   * @param overrideConfig 临时覆盖的配置
   * @returns LLM 响应结果
   */
  public async call(
    messages: ChatMessage[],
    overrideConfig?: Partial<LLMConfig>,
  ): Promise<LLMResponse> {
    if (!this.config.enabled) {
      throw new Error("LLM 未启用");
    }
    if (!this.isConfigured()) {
      throw new Error("LLM 配置未初始化");
    }
    // 合并配置
    const config = { ...this.config, ...overrideConfig };

    // 验证配置
    if (!config.baseUrl || !config.apiKey || !config.model) {
      throw new Error("LLM 配置不完整，请先初始化 baseUrl、apiKey 和 model");
    }

    // 验证消息
    if (!messages || messages.length === 0) {
      throw new Error("消息列表不能为空");
    }

    // 构建 API URL
    let apiUrl = config.baseUrl;
    // 如果 URL 不包含 /chat/completions，自动添加
    if (!apiUrl.includes('/chat/completions')) {
      apiUrl = apiUrl.replace(/\/$/, '') + '/chat/completions';
    }

    // 构建请求体
    const requestBody = {
      model: config.model,
      messages,
      temperature: config.temperature ?? 1,
      max_tokens: config.maxTokens,
      stop: config.stop,
      top_p: config.topP ?? 1,
      frequency_penalty: config.frequencyPenalty ?? 0,
      presence_penalty: config.presencePenalty ?? 0,
    };

    try {
      // 发送请求
      const response = await fetch(apiUrl, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${config.apiKey}`,
        },
        body: JSON.stringify(requestBody),
      });

      // 检查响应状态
      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        const errorMessage =
          errorData.error?.message ||
          `HTTP ${response.status}: ${response.statusText}`;
        throw new Error(errorMessage);
      }

      // 解析响应
      const data: LLMResponse = await response.json();

      return data;
    } catch (error) {
      // 处理网络错误
      if (error instanceof TypeError && error.message.includes("fetch")) {
        throw new Error("网络请求失败，请检查网络连接和 API URL");
      }

      // 重新抛出其他错误
      throw error;
    }
  }

  /**
   * 简化调用：只需传入用户提示词
   * 使用配置的系统提示词和模型
   * @param userPrompt 用户提示词
   * @param options 可选配置
   * @returns 生成的文本内容
   *
   * @example
   * ```typescript
   * const llm = LLMService.getInstance();
   * llm.initialize({
   *   baseUrl: 'https://api.openai.com',
   *   apiKey: 'sk-xxx',
   *   model: 'gpt-3.5-turbo'
   * });
   *
   * const response = await llm.chat('分析这道题的错误原因');
   * console.log(response);
   * ```
   */
  public async chat(
    // 用户提示词或者符合格式的对话历史
    userPrompt: string | ChatMessage[],
    options?: {
      /** 临时覆盖的配置 */
      config?: Partial<LLMConfig>;
    },
  ): Promise<string> {
    if (!this.config.enabled) {
      throw new Error("LLM 未启用");
    }
    // 合并配置
    const config = { ...this.config, ...options?.config };

    // 验证配置
    if (!config.baseUrl || !config.apiKey || !config.model) {
      throw new Error("LLM 配置不完整，请先初始化 baseUrl、apiKey 和 model");
    }

    // 准备消息（每次都重新构建）
    let messages: ChatMessage[] = [];
    if (typeof userPrompt === "string") {
      messages = [
        {
          role: "system",
          content: config.systemPrompt || "",
        },
        {
          role: "user",
          content: userPrompt,
        },
      ];
    } else if (
      Array.isArray(userPrompt) &&
      userPrompt.length > 0 &&
      userPrompt[0].role === "user"
    ) {
      messages = [
        {
          role: "system",
          content: config.systemPrompt || "",
        },
        ...userPrompt,
      ];
    } else {
      messages = userPrompt;
    }

    // 调用 LLM
    const response = await this.call(messages);

    // 检查响应
    if (!response.choices || response.choices.length === 0) {
      throw new Error("LLM 未返回任何响应");
    }

    return response.choices[0].message.content;
  }
}

// ==================== 全局单例导出 ====================

/**
 * 全局 LLM 服务单例
 *
 * @example
 * ```typescript
 * import { llm } from '@/services/llm';
 *
 * // 初始化配置
 * llm.initialize({
 *   baseUrl: 'https://api.openai.com',
 *   apiKey: 'sk-xxx',
 *   model: 'gpt-3.5-turbo',
 *   systemPrompt: '你是一个错题分析助手'
 * });
 *
 * // 简化调用
 * const response = await llm.chat('分析这道数学题的错误原因');
 * console.log(response);
 *
 * // 修改配置
 * llm.config.temperature = 0.7;
 * llm.config.maxTokens = 1000;
 * ```
 */
export const llm = LLMService.getInstance();
