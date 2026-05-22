import fs from 'node:fs/promises';
import path from 'node:path';
import type { PresentationSummary } from '../../../../shared/src/types';
import { OutlineRepo } from './outline-repo';

export class PresentationRepo {
  private dataDir: string;
  private outlineRepo: OutlineRepo;

  constructor(dataDir: string) {
    this.dataDir = dataDir;
    this.outlineRepo = new OutlineRepo(dataDir);
  }

  async list(): Promise<PresentationSummary[]> {
    try {
      await fs.access(this.dataDir);
    } catch {
      return [];
    }

    const entries = await fs.readdir(this.dataDir, { withFileTypes: true });
    const summaries: PresentationSummary[] = [];

    for (const entry of entries) {
      if (!entry.isDirectory()) continue;
      const slug = entry.name;
      try {
        const outline = await this.outlineRepo.read(slug);
        const stat = await fs.stat(path.join(this.dataDir, slug));
        summaries.push({
          slug: outline.slug ?? slug,
          title: outline.title ?? slug,
          slideCount: outline.slides?.length ?? 0,
          totalCost: outline.totalCost ?? 0,
          createdAt: stat.birthtime.toISOString(),
          updatedAt: stat.mtime.toISOString(),
        });
      } catch (err) {
        console.error(`[PresentationRepo] Failed to read outline for "${slug}":`, err);
        throw err;
      }
    }

    return summaries;
  }

  async create(slug: string, title: string): Promise<{ slug: string; title: string }> {
    const dirPath = path.join(this.dataDir, slug);
    await fs.mkdir(dirPath, { recursive: true });
    await this.outlineRepo.init(slug, title);
    return { slug, title };
  }

  async exists(slug: string): Promise<boolean> {
    try {
      await this.outlineRepo.read(slug);
      return true;
    } catch {
      return false;
    }
  }

  async delete(slug: string): Promise<void> {
    const dirPath = path.join(this.dataDir, slug);
    await fs.rm(dirPath, { recursive: true, force: true });
  }

  getSlugDir(slug: string): string {
    return path.join(this.dataDir, slug);
  }
}
