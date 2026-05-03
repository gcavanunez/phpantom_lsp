<?php

namespace App\Models;

/**
 * @template TKey of array-key
 * @template TModel
 * @extends \Illuminate\Database\Eloquent\Collection<TKey, TModel>
 */
class ReviewCollection extends \Illuminate\Database\Eloquent\Collection
{
    /** @return array<TKey, TModel> */
    public function topRated(): array { return []; }

    /** @return float */
    public function averageRating(): float { return 0.0; }
}
