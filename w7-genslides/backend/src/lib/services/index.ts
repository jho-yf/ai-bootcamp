import path from 'node:path';
import { ImageGenerator } from '../ai/image-generator';
import { PresentationRepo } from '../storage/presentation-repo';
import { OutlineRepo } from '../storage/outline-repo';
import { ImageRepo } from '../storage/image-repo';
import { PresentationService } from './presentation-service';
import { SlideService } from './slide-service';
import { GenerateService } from './generate-service';
import { StyleService } from './style-service';

const DATA_DIR = path.resolve(process.env.DATA_DIR ?? './genslides');
const API_KEY = process.env.GOOGLE_AI_API_KEY ?? '';
const MODEL = process.env.GOOGLE_AI_MODEL ?? 'gemini-2.0-flash-exp';
const BASE_URL = process.env.GOOGLE_AI_BASE_URL || undefined;

const presentationRepo = new PresentationRepo(DATA_DIR);
const outlineRepo = new OutlineRepo(DATA_DIR);
const imageRepo = new ImageRepo(DATA_DIR);
const imageGenerator = new ImageGenerator(API_KEY, MODEL, BASE_URL);

export const presentationService = new PresentationService(presentationRepo, outlineRepo);
export const slideService = new SlideService(outlineRepo);
export const generateService = new GenerateService(outlineRepo, imageRepo, imageGenerator, slideService);
export const styleService = new StyleService(outlineRepo, imageRepo, imageGenerator);
export { outlineRepo, imageRepo };
