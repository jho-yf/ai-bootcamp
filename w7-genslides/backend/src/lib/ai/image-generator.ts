import { GoogleGenAI } from '@google/genai';

export interface GeneratedImage {
  buffer: Buffer;
  cost: number;
}

export class ImageGenerator {
  private genai: GoogleGenAI;
  private model: string;

  constructor(apiKey: string, model: string, baseUrl?: string) {
    const opts: { apiKey: string; httpOptions?: { baseUrl: string } } = { apiKey };
    if (baseUrl) {
      opts.httpOptions = { baseUrl };
    }
    this.genai = new GoogleGenAI(opts);
    this.model = model;
  }

  async generate(prompt: string, referenceImage?: Buffer): Promise<GeneratedImage> {
    const parts: Array<{ text: string } | { inlineData: { mimeType: string; data: string } }> = [
      { text: prompt },
    ];

    if (referenceImage) {
      parts.push({
        inlineData: {
          mimeType: 'image/jpeg',
          data: referenceImage.toString('base64'),
        },
      });
    }

    const response = await this.genai.models.generateContent({
      model: this.model,
      contents: [{ role: 'user', parts }],
      config: {
        responseModalities: ['TEXT', 'IMAGE'],
      },
    });

    const candidates = response.candidates ?? [];
    if (candidates.length === 0) {
      throw new Error('No candidates returned from AI model');
    }

    const contentParts = candidates[0].content?.parts ?? [];
    for (const part of contentParts) {
      if (part.inlineData?.data) {
        const buffer = Buffer.from(part.inlineData.data, 'base64');
        return { buffer, cost: this.estimateCost(prompt) };
      }
    }

    throw new Error('No image found in AI response');
  }

  async generateMultiple(
    prompts: string[],
    referenceImage?: Buffer,
    onProgress?: (index: number, total: number) => void,
  ): Promise<GeneratedImage[]> {
    const results: GeneratedImage[] = [];

    for (let i = 0; i < prompts.length; i++) {
      onProgress?.(i, prompts.length);
      const result = await this.generate(prompts[i], referenceImage);
      results.push(result);
    }

    return results;
  }

  private estimateCost(prompt: string): number {
    const baseCost = 0.02;
    const lengthFactor = Math.ceil(prompt.length / 100) * 0.005;
    return Math.round((baseCost + lengthFactor) * 100) / 100;
  }
}
