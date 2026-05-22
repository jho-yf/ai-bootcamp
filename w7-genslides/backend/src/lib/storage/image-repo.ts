import fs from 'node:fs/promises';
import path from 'node:path';

export class ImageRepo {
  private dataDir: string;

  constructor(dataDir: string) {
    this.dataDir = dataDir;
  }

  private imagesDir(slug: string): string {
    return path.join(this.dataDir, slug, 'images');
  }

  async save(slug: string, filename: string, buffer: Buffer): Promise<string> {
    const dir = this.imagesDir(slug);
    await fs.mkdir(dir, { recursive: true });
    const filePath = path.join(dir, filename);
    await fs.writeFile(filePath, buffer);
    return filePath;
  }

  async read(slug: string, filename: string): Promise<Buffer | null> {
    const filePath = path.join(this.imagesDir(slug), filename);
    try {
      return await fs.readFile(filePath);
    } catch {
      return null;
    }
  }

  async delete(slug: string, filename: string): Promise<void> {
    const filePath = path.join(this.imagesDir(slug), filename);
    await fs.unlink(filePath);
  }

  async exists(slug: string, filename: string): Promise<boolean> {
    try {
      await fs.access(path.join(this.imagesDir(slug), filename));
      return true;
    } catch {
      return false;
    }
  }
}
