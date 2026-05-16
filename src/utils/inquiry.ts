import { getBooks, getSubjects } from "../apis";
import { llm } from "../services";
import { ImageContent, TextContent } from "../services";
import { QuestionType } from "../types";


// ==================== 提示词构造辅助函数 ====================
const getExistingSubjects = async (): Promise<string[]> => {
  try {
    const data = await getSubjects();
    return data.map(s => s.name);
  } catch (error) {
    console.error('获取科目失败：', error);
    return [];
  }
}

// ==================== 类型定义 ====================

/**
 * 输出格式类型
 */
export type OutputFormat = "text" | "json";

/**
 * 带标记的提示词项
 */
export interface PromptItem {
  /** 提示词内容 */
  text?: string;
  /** 标记/标签，用于标识 prompt 的类型和用途 */
  tag: string;
  /** 是否必需执行 */
  required?: boolean;
  /** 顺序权重，数字越小越先执行 */
  priority?: number;
  /** 输出格式（默认 json） */
  outputFormat?: OutputFormat;
  /** JSON Schema（当 outputFormat 为 json 时使用） */
  jsonSchema?: string;
}

/**
 * AI 返回的带标记的结果
 */
export interface TaggedResult {
  /** 对应的 prompt 标记 */
  tag: string;
  /** AI 返回的原始内容 */
  content: string;
  /** 是否执行成功 */
  success: boolean;
  /** 解析后的 JSON 数据（如果 outputFormat 为 json） */
  parsedContent?: any;
}

// ==================== 存储键 ====================
const CUSTOM_PROMPTS_STORAGE_KEY = 'custom_ai_prompts';

// ==================== 默认提示词模板 ====================

const DEFAULT_PROMPTS = {
  subject: `这是一些题目和答案的图片，请观察全部图片.
目前存在的科目有: {{existingSubjects}}.
如果题目所处科目位于其中，请给出科目名称;
否则，请问这些图片中的错题属于哪个科目？`,
  question_text: `**请提取图片中的题干内容，以 Markdown 格式返回.**
这是一些题目和答案的图片，注意区分题干，手写作答，和答案的图片.
其中，请观察带有题干信息的图片 (一般是前一张或若干张图片，题干包含问题，选项，补充信息，示意图等).
题干只是学生做题时可以看见的部分，不包括图片中的答案和手写信息.
注意公式使用 \`$\` 或者 \`$$\` 包裹
请提取图片中的题干内容.`,
  analysis: `这是一些题目和答案的图片，请观察全部图片.
请分析错题的错误原因.
你无需照抄图片内容，只需要切中要害进行分析即可，或者给更高深而精悍的点拨
以 Markdown 格式返回.`,
  question_type: `这是一些题目和答案的图片，请观察带有题干信息的图片 (一般是前一张或若干张图片).
全部的题型有:${Object.values(QuestionType).join(", ")}
请在以上题型中选择最适合的题型.`,
  answer: `这是一些题目和答案的图片，请观察全部图片，并区分题干，手写作答，答案的图片.
如果存在答案的图片，请提取图片中的答案和解析内容，并以 Markdown 格式返回;
否则，尝试作答并给出该题的正确答案和解析，标注 **"AI 生成答案"**，以 Markdown 格式返回.
注意 **只** 输出图片中 *答案* 和 *解析* 部分的原样内容，不用给出 *题干*;
注意公式使用 \`$\` 或者 \`$$\` 包裹`,
};

/**
 * 获取自定义提示词
 */
const getCustomPrompts = (): Record<string, string> => {
  try {
    const stored = localStorage.getItem(CUSTOM_PROMPTS_STORAGE_KEY);
    return stored ? JSON.parse(stored) : {};
  } catch (error) {
    console.error('加载自定义提示词失败:', error);
    return {};
  }
};

/**
 * 根据 tag 获取提示词文本（优先使用自定义，否则用默认）
 */
const getPromptText = async (tag: string): Promise<string> => {
  const customPrompts = getCustomPrompts();

  // 如果是可定制的提示词，优先使用自定义的
  if (['question_text', 'answer', 'analysis'].includes(tag)) {
    if (customPrompts[tag]) {
      return customPrompts[tag];
    }
  }

  // 返回默认提示词（subject 和 question_type 需要动态注入）
  if (tag in DEFAULT_PROMPTS) {
    let dprompt =  DEFAULT_PROMPTS[tag as keyof typeof DEFAULT_PROMPTS];
    if (tag === 'subject') {
      let existingSubjects = await getExistingSubjects();
      dprompt = dprompt.replace('{{existingSubjects}}', existingSubjects.join(', '));
    }
    return dprompt;
  }

  return '';
};

// ==================== Prompts 配置 ====================

/**
 * 带标记的提示词列表
 * 可以通过 tag 来筛选、识别和映射结果
 * 注：实际提示词文本通过 getPromptText() 动态获取
 */
const prompts: PromptItem[] = [
  {
    tag: "subject",
    required: true,
    priority: 1,
    outputFormat: "json",
    jsonSchema: JSON.stringify({
      type: "object",
      properties: {
        subject: {
          type: "string",
          description: "科目名称"
        },
      },
      required: ["subject"]
    }, null, 2)
  },
  {
    tag: "question_text",
    required: true,
    priority: 1,
    outputFormat: "text",
    jsonSchema: ""
  },
  {
    tag: "analysis",
    required: true,
    priority: 3,
    outputFormat: "text",
    jsonSchema: ""
  },
  {
    tag: "question_type",
    required: true,
    priority: 2,
    outputFormat: "json",
    jsonSchema: JSON.stringify({
      type: "object",
      properties: {
        questionType: {
          type: "string",
          description: "题型"
        }
      },
      required: ["questionType"]
    })
  },
  {
    tag: "answer",
    required: true,
    priority: 2,
    outputFormat: "text",
    jsonSchema: ""
  },
];

// ==================== 导出函数 ====================

/**
 * 验证 base64 图片数据的完整性
 */
const validateBase64Image = (base64: string): boolean => {
  if (!base64 || typeof base64 !== 'string') {
    return false;
  }

  // 检查是否包含正确的 data URI 前缀
  const dataUriPrefix = base64.substring(0, 30);
  if (!dataUriPrefix.startsWith('data:image/') || !dataUriPrefix.includes(';base64,')) {
    console.warn('Base64 图片缺少正确的 data URI 前缀:', dataUriPrefix);
    return false;
  }

  // 检查 base64 数据是否有效
  const base64Data = base64.split(',')[1];
  if (!base64Data || base64Data.length < 100) {
    console.warn('Base64 数据过短，可能不完整');
    return false;
  }

  // 尝试解码验证
  try {
    // 只验证前几个字符，避免性能问题
    atob(base64Data.substring(0, 100));
    return true;
  } catch (e) {
    console.warn('Base64 数据格式无效:', e);
    return false;
  }
};

/**
 * 使用 AI 提取错题信息（带标记）
 * @param imgs 图片 base64 数组
 * @param tags 需要执行的 prompt 标签列表，不传则执行所有必需的 prompt
 * @returns 带标记的结果数组
 */
export const inquiryAIAddInfo = async (
  imgs: string[],
  tags?: string[],
): Promise<TaggedResult[]> => {
  const results: TaggedResult[] = [];

  // 调试日志：检查传入的图片数组
  console.log("=== inquiryAIAddInfo 调试 ===");
  console.log("传入的图片数量:", imgs.length);
  console.log("第一个图片数据（前 100 字符）:", imgs[0] ? imgs[0].substring(0, 100) : "无");
  console.log("=========================");

  // 验证图片数据
  const validImages = imgs.filter(img => {
    const isValid = validateBase64Image(img);
    if (!isValid) {
      console.error('图片数据验证失败，已跳过:', img.substring(0, 50));
    }
    return isValid;
  });

  if (validImages.length === 0) {
    console.error('没有有效的图片数据！');
    return results;
  }

  console.log(`验证通过的图片数量：${validImages.length}/${imgs.length}`);

  // 构建 ImageContent 数组
  const imgContents: ImageContent[] = validImages.map((img) => ({
    type: "image_url",
    image_url: {
      url: img,
    },
  }));

  console.log("构建的 imgContents:", imgContents);

  // 筛选需要执行的 prompts
  const filteredPrompts = tags
    ? prompts.filter((p) => tags.includes(p.tag))
    : prompts.filter((p) => p.required !== false);

  // 按优先级排序
  filteredPrompts.sort((a, b) => (a.priority || 0) - (b.priority || 0));

  // 遍历执行每个 prompt
  for (const prompt of filteredPrompts) {
    let retryCount = 0;
    const maxRetries = 2;

    while (retryCount <= maxRetries) {
      try {
        // 添加延迟，避免请求过快
        if (retryCount > 0) {
          console.log(`[${prompt.tag}] 第 ${retryCount} 次重试，等待 1 秒...`);
          await new Promise(resolve => setTimeout(resolve, 1000));
        }

        // 获取提示词文本
        const promptText = await getPromptText(prompt.tag);

        // 构建消息内容
        const content: Array<TextContent | ImageContent> = [];

        // 如果要求 JSON 格式，添加格式说明
        if (prompt.outputFormat === "json") {
          content.push({
            type: "text",
            text: `${promptText}\n\n请严格按照以下 JSON Schema 格式返回结果，不要包含任何其他文字说明：\n\n${prompt.jsonSchema}\n\n`,
          });
        } else {
          content.push({
            type: "text",
            text: promptText
          })
        }

        content.push(...imgContents);

        // 调用 AI
        console.log(`[${prompt.tag}] 构建的消息内容:`, content);
        console.log(`[${prompt.tag}] 图片数量:`, imgContents.length);
        const response = await llm.chat([
          {
            role: "user",
            content,
          },
        ]);
        console.log(`[${prompt.tag}] AI 响应:`, response);

        // 解析响应
        let parsedContent: any = undefined;
        if (prompt.outputFormat === "json") {
          try {
            console.log(`[${prompt.tag}] 尝试解析 JSON...`);
            // 尝试提取 JSON（可能被包裹在代码块中）
            const jsonMatch = response.match(/```json\s*([\s\S]*?)\s*```/) ||
              response.match(/```\s*([\s\S]*?)\s*```/) ||
              [null, response];

            const jsonStr = jsonMatch[1].trim();
            parsedContent = JSON.parse(jsonStr);
            console.log(`[${prompt.tag}] 解析后的 JSON:`, parsedContent);
          } catch (parseError) {
            console.error(`解析 JSON 失败 [${prompt.tag}]:`, parseError);
            // JSON 解析失败，标记为失败但不抛出异常
            results.push({
              tag: prompt.tag,
              content: response,
              success: false,
            });
            break;
          }
        } else {
          console.log(`[${prompt.tag}] 纯文本结果:`, response);
          parsedContent = response.replace(/\\\\/g, "\\")
            .replace(/\$\$/g, "<math-block>")
            .replace(/\$/g, "<math-inline>")
            .replace(/<math-block>/g, " $$ ")
            .replace(/<math-inline>/g, " $ ");
          console.log(`替换后结果：${parsedContent}`);
        }

        // 保存结果
        results.push({
          tag: prompt.tag,
          content: response,
          success: true,
          parsedContent,
        });
        break; // 成功，跳出重试循环
      } catch (error) {
        console.error(`执行 prompt [${prompt.tag}] 失败:`, error);
        retryCount++;
        if (retryCount > maxRetries) {
          results.push({
            tag: prompt.tag,
            content: "",
            success: false,
          });
        }
      }
    }
  }
  console.log(results);
  return results;
};

/**
 * 获取所有可用的 prompt 标签
 */
export const getPromptTags = (): string[] => {
  return prompts.map((p) => p.tag);
};

/**
 * 根据 tag 获取 prompt 项
 */
export const getPromptByTag = (tag: string): PromptItem | undefined => {
  return prompts.find((p) => p.tag === tag);
};

/**
 * 获取当前使用的提示词文本（仅用于调试/测试）
 */
export const getCurrentPromptText = async (tag: string): Promise<string> => {
  return getPromptText(tag);
};
