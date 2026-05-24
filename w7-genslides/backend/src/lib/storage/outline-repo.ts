import fs from 'node:fs/promises';
import path from 'node:path';
import YAML from 'yaml';
import type { Presentation, Slide } from '../../../../shared/src/types';

const WRITE_QUEUE = new Map<string, Promise<void>>();

export class OutlineRepo {
  private dataDir: string;

  constructor(dataDir: string) {
    this.dataDir = dataDir;
  }

  private outlinePath(slug: string): string {
    return path.join(this.dataDir, slug, 'outline.yaml');
  }

  async read(slug: string): Promise<Presentation> {
    const filePath = this.outlinePath(slug);
    const raw = await fs.readFile(filePath, 'utf-8');
    return this.parseYaml(raw, slug);
  }

  async write(slug: string, data: Presentation): Promise<void> {
    const prev = WRITE_QUEUE.get(slug) ?? Promise.resolve();
    const next = prev.then(() => this._write(slug, data));
    WRITE_QUEUE.set(slug, next.catch(() => {}));
    await next;
  }

  private async _write(slug: string, data: Presentation): Promise<void> {
    const filePath = this.outlinePath(slug);
    const dir = path.dirname(filePath);
    await fs.mkdir(dir, { recursive: true });

    const tmpPath = `${filePath}.tmp`;
    const content = YAML.stringify(data);
    await fs.writeFile(tmpPath, content, 'utf-8');
    await fs.rename(tmpPath, filePath);
  }

  async init(slug: string, title: string): Promise<void> {
    const data: Presentation = {
      slug,
      title,
      style: { prompt: '', candidates: [], referenceImage: null },
      slides: [],
      totalCost: 0,
    };
    await this.write(slug, data);
  }

  private parseYaml(raw: string, slug: string): Presentation {
    const obj = YAML.parse(raw) as Record<string, unknown>;
    const style = (obj.style ?? { prompt: '', candidates: [], referenceImage: null }) as Record<string, unknown>;
    const slidesRaw = obj.slides ?? [];
    if (!Array.isArray(slidesRaw)) {
      throw new Error(`Invalid YAML: "slides" must be an array, got ${typeof slidesRaw} in ${slug}/outline.yaml`);
    }
    const candidatesRaw = style.candidates ?? [];
    if (!Array.isArray(candidatesRaw)) {
      throw new Error(`Invalid YAML: "style.candidates" must be an array, got ${typeof candidatesRaw} in ${slug}/outline.yaml`);
    }
    return {
      slug: typeof obj.slug === 'string' ? obj.slug : slug,
      title: typeof obj.title === 'string' ? obj.title : slug,
      style: {
        prompt: typeof style.prompt === 'string' ? style.prompt : '',
        candidates: candidatesRaw.map((c) => String(c)),
        referenceImage: typeof style.referenceImage === 'string' ? style.referenceImage : null,
      },
      slides: slidesRaw.map((s: Record<string, unknown>) => {
        const imagesRaw = s.images ?? [];
        if (!Array.isArray(imagesRaw)) {
          throw new Error(`Invalid YAML: "slides[].images" must be an array, got ${typeof imagesRaw} in ${slug}/outline.yaml`);
        }
        return {
          sid: s.sid as string,
          content: (s.content as string) ?? '',
          images: imagesRaw.map((img: unknown) => {
            const i = img as Record<string, unknown>;
            return {
              filename: i.filename as string,
              textHash: i.textHash as string,
              createdAt: i.createdAt as string,
              cost: (i.cost as number) ?? 0,
            };
          }),
          activeImageIndex: (s.activeImageIndex as number) ?? 0,
        } as Slide;
      }),
      totalCost: (obj.totalCost as number) ?? 0,
    };
  }
}
