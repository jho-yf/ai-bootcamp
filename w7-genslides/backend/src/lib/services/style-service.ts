import { blake3 } from 'hash-wasm';
import type { StyleConfig } from '../../../../shared/src/types';
import { ImageGenerator } from '../ai/image-generator';
import { OutlineRepo } from '../storage/outline-repo';
import { ImageRepo } from '../storage/image-repo';

export class StyleService {
  constructor(
    private outlineRepo: OutlineRepo,
    private imageRepo: ImageRepo,
    private imageGenerator: ImageGenerator,
  ) {}

  async generateCandidates(slug: string, prompt: string): Promise<{ candidates: string[]; prompt: string; errors: string[] }> {
    const data = await this.outlineRepo.read(slug);

    // Delete old candidates that aren't the selected style
    for (const old of data.style.candidates) {
      if (old !== data.style.referenceImage) {
        await this.imageRepo.delete(slug, old).catch(() => {});
      }
    }

    const candidates: string[] = [];
    const errors: string[] = [];

    const textHash = await blake3(prompt);
    const filename = `style-${textHash}.jpg`;

    try {
      const result = await this.imageGenerator.generate(prompt);
      await this.imageRepo.save(slug, filename, result.buffer);
      candidates.push(filename);
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error);
      console.error(`[StyleService] Failed to generate style image:`, msg);
      errors.push(msg);
    }

    data.style.candidates = candidates;
    data.style.prompt = prompt;
    await this.outlineRepo.write(slug, data);

    return { candidates, prompt, errors };
  }

  async selectStyle(slug: string, referenceImage: string | null): Promise<StyleConfig> {
    const data = await this.outlineRepo.read(slug);

    // Delete unselected candidates from disk
    for (const candidate of data.style.candidates) {
      if (candidate !== referenceImage) {
        await this.imageRepo.delete(slug, candidate).catch(() => {});
      }
    }

    data.style.referenceImage = referenceImage;
    data.style.candidates = [];
    await this.outlineRepo.write(slug, data);
    return data.style;
  }

  async needsGuide(slug: string): Promise<boolean> {
    const data = await this.outlineRepo.read(slug);
    return data.style.referenceImage === null;
  }

  async cleanupCandidates(slug: string): Promise<void> {
    const data = await this.outlineRepo.read(slug);
    for (const candidate of data.style.candidates) {
      if (candidate !== data.style.referenceImage) {
        await this.imageRepo.delete(slug, candidate).catch(() => {});
      }
    }
    data.style.candidates = [];
    await this.outlineRepo.write(slug, data);
  }
}
