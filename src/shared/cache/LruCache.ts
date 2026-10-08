type CacheEntry<T> = { value: T; weight: number };

export class LruCache<K, V> {
  private readonly entries = new Map<K, CacheEntry<V>>();
  private totalWeight = 0;

  constructor(
    private readonly maxEntries: number,
    private readonly maxWeight = Number.POSITIVE_INFINITY,
    private readonly weigh: (value: V) => number = () => 1,
  ) {}

  get(key: K): V | undefined {
    const entry = this.entries.get(key);
    if (!entry) return undefined;
    this.entries.delete(key);
    this.entries.set(key, entry);
    return entry.value;
  }

  set(key: K, value: V) {
    const existing = this.entries.get(key);
    if (existing) {
      this.totalWeight -= existing.weight;
      this.entries.delete(key);
    }
    const weight = Math.max(0, this.weigh(value));
    this.entries.set(key, { value, weight });
    this.totalWeight += weight;
    this.trim();
  }

  deleteWhere(predicate: (key: K) => boolean) {
    for (const [key, entry] of this.entries) {
      if (!predicate(key)) continue;
      this.entries.delete(key);
      this.totalWeight -= entry.weight;
    }
  }

  private trim() {
    while (this.entries.size > this.maxEntries || this.totalWeight > this.maxWeight) {
      const oldest = this.entries.entries().next().value as [K, CacheEntry<V>] | undefined;
      if (!oldest) break;
      this.entries.delete(oldest[0]);
      this.totalWeight -= oldest[1].weight;
    }
  }
}
