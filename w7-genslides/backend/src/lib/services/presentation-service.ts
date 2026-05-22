import type { Presentation, PresentationSummary } from '../../../../shared/src/types';
import { AppError, ErrorCodes } from '../../types/errors';
import { PresentationRepo } from '../storage/presentation-repo';
import { OutlineRepo } from '../storage/outline-repo';

export class PresentationService {
  constructor(
    private presentationRepo: PresentationRepo,
    private outlineRepo: OutlineRepo,
  ) {}

  async list(): Promise<PresentationSummary[]> {
    return this.presentationRepo.list();
  }

  async create(title: string): Promise<{ slug: string; title: string }> {
    if (!title) {
      throw new AppError(ErrorCodes.INVALID_REQUEST, 400, 'title is required');
    }

    const slug = this.generateSlug(title);
    const exists = await this.presentationRepo.exists(slug);
    if (exists) {
      throw new AppError(ErrorCodes.INVALID_REQUEST, 409, `Presentation "${slug}" already exists`);
    }

    return this.presentationRepo.create(slug, title);
  }

  private generateSlug(title: string): string {
    const base = title
      .toLowerCase()
      .replace(/[^a-z0-9一-龥]+/g, '-')
      .replace(/^-|-$/g, '');
    const timestamp = Date.now().toString(36);
    return `${base}-${timestamp}`;
  }

  async get(slug: string): Promise<Presentation> {
    const exists = await this.presentationRepo.exists(slug);
    if (!exists) {
      throw new AppError(ErrorCodes.PRESENTATION_NOT_FOUND, 404, `Presentation "${slug}" not found`);
    }

    return this.outlineRepo.read(slug);
  }

  async update(slug: string, title: string): Promise<{ slug: string; title: string }> {
    const exists = await this.presentationRepo.exists(slug);
    if (!exists) {
      throw new AppError(ErrorCodes.PRESENTATION_NOT_FOUND, 404, `Presentation "${slug}" not found`);
    }

    const data = await this.outlineRepo.read(slug);
    data.title = title;
    await this.outlineRepo.write(slug, data);
    return { slug, title };
  }

  async delete(slug: string): Promise<void> {
    const exists = await this.presentationRepo.exists(slug);
    if (!exists) {
      throw new AppError(ErrorCodes.PRESENTATION_NOT_FOUND, 404, `Presentation "${slug}" not found`);
    }

    await this.presentationRepo.delete(slug);
  }
}
