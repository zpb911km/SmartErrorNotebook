import { invoke } from "@tauri-apps/api/core";
import { mockErrorQuestion1 } from "../types/mock";
import { ErrorQuestion } from "../types";

async function greetWithName(name: string): Promise<string> {
    return await invoke("greet", { name });
}

async function getErrorQuestionByName(name: string): Promise<ErrorQuestion> {
    // 先使用假数据
    return mockErrorQuestion1;
}

export { greetWithName, getErrorQuestionByName };