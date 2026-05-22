import { NextRequest } from 'next/server';
import { generateService } from '../../../../../../src/lib/services';
import type { GenerateProgress } from '../../../../../../../shared/src/types';

export async function POST(
  _request: NextRequest,
  { params }: { params: Promise<{ slug: string; sid: string }> },
) {
  const { slug, sid } = await params;

  const stream = new ReadableStream({
    async start(controller) {
      const send = (event: string, data: GenerateProgress) => {
        controller.enqueue(`event: ${event}\ndata: ${JSON.stringify(data)}\n\n`);
      };

      try {
        send('progress', { status: 'generating', progress: 0 });

        const image = await generateService.generateSlideImage(slug, sid);

        send('complete', { status: 'complete', image });
        controller.close();
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Generation failed';
        send('error', { status: 'error', error: message });
        controller.close();
      }
    },
  });

  return new Response(stream, {
    headers: {
      'Content-Type': 'text/event-stream',
      'Cache-Control': 'no-cache',
      Connection: 'keep-alive',
    },
  });
}
