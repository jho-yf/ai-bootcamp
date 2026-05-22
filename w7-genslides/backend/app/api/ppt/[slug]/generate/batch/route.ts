import { NextRequest } from 'next/server';
import { generateService } from '../../../../../../src/lib/services';
import type { BatchGenerateProgress } from '../../../../../../../shared/src/types';
import type { BatchGenerateBody } from '../../../../../../src/types/api';

export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ slug: string }> },
) {
  const { slug } = await params;
  const body: BatchGenerateBody = await request.json();

  const stream = new ReadableStream({
    async start(controller) {
      const send = (event: string, data: BatchGenerateProgress) => {
        controller.enqueue(`event: ${event}\ndata: ${JSON.stringify(data)}\n\n`);
      };

      try {
        const { totalCost } = await generateService.batchGenerate(slug, body.sids, (sid, image) => {
          send('complete', { sid, status: 'complete', image });
        });

        controller.enqueue(`event: done\ndata: ${JSON.stringify({ totalCost })}\n\n`);
        controller.close();
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Batch generation failed';
        send('error', { sid: '', status: 'error', error: message });
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
