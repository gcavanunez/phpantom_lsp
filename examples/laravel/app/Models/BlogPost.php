<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;

class BlogPost extends Model
{
    public function getTitle(): string { return ''; }
    public function getSlug(): string { return ''; }

    /** @return \Illuminate\Database\Eloquent\Relations\BelongsTo<BlogAuthor, covariant $this> */
    public function author(): mixed { return $this->belongsTo(BlogAuthor::class); }
}
