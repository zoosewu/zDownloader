/** Selecting downloads like files: click, Ctrl click, Shift click, right click. */
import { describe, expect, test } from 'vitest';
import { click, clickMode, focused, forMenu, noSelection, within } from '../utils/selection';

const shown = ['a', 'b', 'c', 'd', 'e'];

describe('download selection', () => {
  test('a click selects only that download; clicking it again deselects it', () => {
    const one = click(noSelection, 'b', 'only', shown);
    expect(one).toEqual({ ids: ['b'], anchor: 'b' });
    expect(click(one, 'b', 'only', shown)).toEqual(noSelection);
    expect(click({ ids: ['a', 'b'], anchor: 'a' }, 'b', 'only', shown)).toEqual({ ids: ['b'], anchor: 'b' });
  });

  test('Ctrl adds and removes one at a time', () => {
    let s = click(noSelection, 'b', 'toggle', shown);
    s = click(s, 'd', 'toggle', shown);
    expect(s.ids).toEqual(['b', 'd']);
    s = click(s, 'b', 'toggle', shown);
    expect(s.ids).toEqual(['d']);
    expect(focused(s)).toBe('d');
  });

  test('Shift selects the range from the last clicked, in the shown order, either direction', () => {
    const start = click(noSelection, 'b', 'only', shown);
    expect(click(start, 'd', 'range', shown)).toEqual({ ids: ['b', 'c', 'd'], anchor: 'b' });
    const back = click(click(noSelection, 'd', 'only', shown), 'a', 'range', shown);
    expect(back.ids).toEqual(['a', 'b', 'c', 'd']);
    // A second Shift click moves the end, not the start.
    expect(click(back, 'e', 'range', shown).ids).toEqual(['d', 'e']);
    // Nothing to start from: just that one.
    expect(click(noSelection, 'c', 'range', shown).ids).toEqual(['c']);
  });

  test('modifier keys', () => {
    expect(clickMode({ ctrlKey: false, metaKey: false, shiftKey: false })).toBe('only');
    expect(clickMode({ ctrlKey: true, metaKey: false, shiftKey: false })).toBe('toggle');
    expect(clickMode({ ctrlKey: false, metaKey: true, shiftKey: false })).toBe('toggle');
    expect(clickMode({ ctrlKey: true, metaKey: false, shiftKey: true })).toBe('range');
  });

  test('right click keeps a selection it is part of, otherwise selects only that row', () => {
    const many = { ids: ['a', 'c'], anchor: 'c' };
    expect(forMenu(many, 'a')).toBe(many);
    expect(forMenu(many, 'b')).toEqual({ ids: ['b'], anchor: 'b' });
  });

  test('downloads that leave the view leave the selection', () => {
    const s = { ids: ['a', 'c', 'e'], anchor: 'c' };
    expect(within(s, shown)).toBe(s);
    expect(within(s, ['a', 'e'])).toEqual({ ids: ['a', 'e'], anchor: 'e' });
    expect(within(s, ['b'])).toEqual(noSelection);
  });
});
