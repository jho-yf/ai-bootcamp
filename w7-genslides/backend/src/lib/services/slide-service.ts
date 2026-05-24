import type { Presentation, Slide } from '../../../../shared/src/types';
import { AppError, ErrorCodes } from '../../types/errors';
import { ImageRepo } from '../storage/image-repo';
import { OutlineRepo } from '../storage/outline-repo';

export class SlideService {
  constructor(
    private outlineRepo: OutlineRepo,
    private imageRepo?: ImageRepo,
  ) {}

  async getPresentationData(slug: string): Promise<{ title: string; style: Presentation['style']; slides: Slide[]; totalCost: number }> {
    const data = await this.outlineRepo.read(slug);
    return {
      title: data.title,
      style: data.style,
      slides: data.slides,
      totalCost: data.totalCost,
    };
  }

  async createSlide(slug: string, content: string, index?: number): Promise<Slide> {
    const data = await this.outlineRepo.read(slug);
    if (data.style.referenceImage === null) {
      throw new AppError(ErrorCodes.INVALID_REQUEST, 400, '请先生成并选择风格图片');
    }
    const sid = `slide-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;

    const newSlide: Slide = {
      sid,
      content,
      images: [],
      activeImageIndex: 0,
    };

    if (index !== undefined && index >= 0 && index <= data.slides.length) {
      data.slides.splice(index, 0, newSlide);
    } else {
      data.slides.push(newSlide);
    }

    await this.outlineRepo.write(slug, data);
    return newSlide;
  }

  async updateSlide(
    slug: string,
    sid: string,
    updates: { content?: string; activeImageIndex?: number },
  ): Promise<{ slide: Slide; hasMatchingImage: boolean }> {
    const data = await this.outlineRepo.read(slug);
    const slide = data.slides.find(s => s.sid === sid);
    if (!slide) {
      throw new AppError(ErrorCodes.SLIDE_NOT_FOUND, 404, `Slide "${sid}" not found`);
    }

    if (updates.content !== undefined) {
      slide.content = updates.content;
    }
    if (updates.activeImageIndex !== undefined) {
      slide.activeImageIndex = updates.activeImageIndex;
    }

    const hasMatchingImage = slide.images.length > 0 && slide.activeImageIndex < slide.images.length;
    await this.outlineRepo.write(slug, data);
    return { slide, hasMatchingImage };
  }

  async deleteSlide(slug: string, sid: string): Promise<void> {
    const data = await this.outlineRepo.read(slug);
    const index = data.slides.findIndex(s => s.sid === sid);
    if (index === -1) {
      throw new AppError(ErrorCodes.SLIDE_NOT_FOUND, 404, `Slide "${sid}" not found`);
    }

    data.slides.splice(index, 1);
    await this.outlineRepo.write(slug, data);
  }

  async deleteSlideImage(slug: string, sid: string, imageIndex: number): Promise<Slide> {
    const data = await this.outlineRepo.read(slug);
    const slide = data.slides.find(s => s.sid === sid);
    if (!slide) {
      throw new AppError(ErrorCodes.SLIDE_NOT_FOUND, 404, `Slide "${sid}" not found`);
    }
    if (!Number.isInteger(imageIndex) || imageIndex < 0 || imageIndex >= slide.images.length) {
      throw new AppError(ErrorCodes.INVALID_REQUEST, 400, '图片不存在');
    }

    const [image] = slide.images.splice(imageIndex, 1);
    if (slide.images.length === 0) {
      slide.activeImageIndex = 0;
    } else if (slide.activeImageIndex >= slide.images.length) {
      slide.activeImageIndex = slide.images.length - 1;
    } else if (imageIndex < slide.activeImageIndex) {
      slide.activeImageIndex -= 1;
    }

    await this.outlineRepo.write(slug, data);
    await this.imageRepo?.delete(slug, image.filename).catch(() => {});
    return slide;
  }

  async reorderSlides(slug: string, orderedSids: string[]): Promise<Slide[]> {
    const data = await this.outlineRepo.read(slug);
    const slideMap = new Map(data.slides.map(s => [s.sid, s]));
    const reordered: Slide[] = [];

    for (const sid of orderedSids) {
      const slide = slideMap.get(sid);
      if (slide) {
        reordered.push(slide);
      }
    }

    data.slides = reordered;
    await this.outlineRepo.write(slug, data);
    return data.slides;
  }

  async addImageToSlide(
    slug: string,
    sid: string,
    image: { filename: string; textHash: string; createdAt: string; cost: number },
  ): Promise<Slide> {
    const data = await this.outlineRepo.read(slug);
    const slide = data.slides.find(s => s.sid === sid);
    if (!slide) {
      throw new AppError(ErrorCodes.SLIDE_NOT_FOUND, 404, `Slide "${sid}" not found`);
    }

    slide.images.push(image);
    slide.activeImageIndex = slide.images.length - 1;
    data.totalCost = Math.round((data.totalCost + image.cost) * 100) / 100;

    await this.outlineRepo.write(slug, data);
    return slide;
  }

  async getSlide(slug: string, sid: string): Promise<Slide> {
    const data = await this.outlineRepo.read(slug);
    const slide = data.slides.find(s => s.sid === sid);
    if (!slide) {
      throw new AppError(ErrorCodes.SLIDE_NOT_FOUND, 404, `Slide "${sid}" not found`);
    }
    return slide;
  }
}
