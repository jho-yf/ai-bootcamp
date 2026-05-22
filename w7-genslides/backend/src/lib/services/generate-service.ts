import { blake3 } from 'hash-wasm';
import type { SlideImage } from '../../../../shared/src/types';
import { AppError, ErrorCodes } from '../../types/errors';
import { ImageGenerator } from '../ai/image-generator';
import { OutlineRepo } from '../storage/outline-repo';
import { ImageRepo } from '../storage/image-repo';
import { SlideService } from './slide-service';

export class GenerateService {
  constructor(
    private outlineRepo: OutlineRepo,
    private imageRepo: ImageRepo,
    private imageGenerator: ImageGenerator,
    private slideService: SlideService,
  ) {}

  async computePromptHash(prompt: string): Promise<string> {
    return blake3(prompt);
  }

  async generateSlideImage(slug: string, sid: string): Promise<SlideImage> {
    const data = await this.outlineRepo.read(slug);
    const slide = data.slides.find(s => s.sid === sid);
    if (!slide) {
      throw new AppError(ErrorCodes.SLIDE_NOT_FOUND, 404, `Slide "${sid}" not found`);
    }

    const prompt = this.buildPrompt(slide.content, data.style.prompt);
    const textHash = await this.computePromptHash(prompt);
    const filename = `${textHash}.jpg`;

    const existing = await this.imageRepo.read(slug, filename);
    if (existing) {
      const image: SlideImage = {
        filename,
        textHash,
        createdAt: new Date().toISOString(),
        cost: 0,
      };
      await this.slideService.addImageToSlide(slug, sid, image);
      return image;
    }

    let refImage: Buffer | undefined;
    if (data.style.referenceImage) {
      const refBuf = await this.imageRepo.read(slug, data.style.referenceImage);
      if (refBuf) refImage = refBuf;
    }

    const result = await this.imageGenerator.generate(prompt, refImage);
    await this.imageRepo.save(slug, filename, result.buffer);

    const image: SlideImage = {
      filename,
      textHash,
      createdAt: new Date().toISOString(),
      cost: result.cost,
    };

    await this.slideService.addImageToSlide(slug, sid, image);
    return image;
  }

  async batchGenerate(
    slug: string,
    sids: string[],
    onSlideComplete: (sid: string, image: SlideImage) => void,
  ): Promise<{ totalCost: number }> {
    let totalCost = 0;

    for (const sid of sids) {
      try {
        const image = await this.generateSlideImage(slug, sid);
        totalCost += image.cost;
        onSlideComplete(sid, image);
      } catch (error) {
        console.error(`[GenerateService] Failed to generate for slide ${sid}:`, error);
      }
    }

    return { totalCost: Math.round(totalCost * 100) / 100 };
  }

  private buildPrompt(content: string, stylePrompt: string): string {
    const parts: string[] = [];
    if (stylePrompt) {
      parts.push(`Style: ${stylePrompt}`);
    }
    parts.push(`Content: ${content}`);
    return parts.join('\n');
  }
}
