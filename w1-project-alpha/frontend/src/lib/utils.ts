import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

/**
 * 合并 Tailwind CSS 类名
 * 使用 clsx 和 tailwind-merge 来智能合并类名，避免冲突
 * 
 * @param inputs - 类名数组或对象
 * @returns 合并后的类名字符串
 * 
 * @example
 * cn("px-2 py-1", "px-4") // => "py-1 px-4" (px-2 被 px-4 覆盖)
 * cn({ "bg-red-500": true, "text-white": false }) // => "bg-red-500"
 */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
