import type { Counts, Queue, Rating } from '@/lib/screen';

export type SampleCard = {
  id: 'cat' | 'station' | 'rain' | 'thursday';
  sound: string;
  queue: Queue;
  intervals: Record<Rating, string>;
};

// A few Latin-script cards: wide characters would break the terminal grid
export const SAMPLE_CARDS: readonly SampleCard[] = [
  {
    id: 'cat',
    sound: 'gato.mp3',
    queue: 'review',
    intervals: { again: '<10m', hard: '2d', good: '5d', easy: '11d' },
  },
  {
    id: 'station',
    sound: 'estacion.mp3',
    queue: 'new',
    intervals: { again: '<1m', hard: '<6m', good: '<10m', easy: '4d' },
  },
  {
    id: 'rain',
    sound: 'llover.mp3',
    queue: 'learn',
    intervals: { again: '<1m', hard: '<6m', good: '1d', easy: '3d' },
  },
  {
    id: 'thursday',
    sound: 'jueves.mp3',
    queue: 'review',
    intervals: { again: '<10m', hard: '3d', good: '8d', easy: '19d' },
  },
];

export const START_COUNTS: Counts = { new: 12, learn: 3, review: 41 };
