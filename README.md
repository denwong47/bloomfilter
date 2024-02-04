# Bloom Filter

A Bloom filter is a space-efficient probabilistic data structure that is used to test whether an element is a member of a set. False positive matches are possible, but false negatives are not – in other words, a query returns either "possibly in set" or "definitely not in set". Elements can be added to the set, but not removed (though this can be addressed with a "counting" filter). The more elements that are added to the set, the larger the probability of false positives.

This particular implementation performs a 128-bit hash based on [Gxhash](https://docs.rs/gxhash/3.0.0/gxhash/) before splitting the hash into N 32-bit integers, where N could be 4 or 8. The filter is then updated with each of these integers. The filter is then checked for membership by hashing the input and checking the corresponding bits in the filter. If all bits are set, the input is considered to be a member of the set. If any of the bits are not set, the input is not a member of the set.

For most typical usages, 4 32-bit integers are sufficient to provide a sufficiently low
false positive rate. However, if a lower false positive rate is required, 8 32-bit integers can be used at the cost of lower overall capacity to 65536, with more hash
collisions as insertions approach this capacity.

This library also provides an implementation of a rolling Bloom Filter window, which is
a rolling window of Bloom Filters that can be used to track the membership of a set of
elements over time. Since the filters are very low in memory, using a stack of them can
allow us to space efficiently track the membership of a set of elements over time,
expiring old elements by dropping the oldest filter in favour of a new one.
