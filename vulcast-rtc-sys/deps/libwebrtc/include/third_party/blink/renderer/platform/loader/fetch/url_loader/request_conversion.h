// Copyright 2020 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#ifndef THIRD_PARTY_BLINK_RENDERER_PLATFORM_LOADER_FETCH_URL_LOADER_REQUEST_CONVERSION_H_
#define THIRD_PARTY_BLINK_RENDERER_PLATFORM_LOADER_FETCH_URL_LOADER_REQUEST_CONVERSION_H_

// This file consists of request conversion functions between blink and network.

#include "base/memory/ref_counted.h"
#include "third_party/blink/renderer/platform/platform_export.h"

namespace network {
class ResourceRequestBody;
struct ResourceRequest;
}  // namespace network

namespace blink {

PLATFORM_EXPORT const char* ImageAcceptHeader();

class ResourceRequestHead;
class ResourceRequestBody;

scoped_refptr<network::ResourceRequestBody> NetworkResourceRequestBodyFor(
    const ResourceRequestBody src_body,
    bool allow_http1_for_streaming_upload);

void PopulateResourceRequest(const ResourceRequestHead& src,
                             ResourceRequestBody src_body,
                             network::ResourceRequest* dest);
}  // namespace blink

#endif  // THIRD_PARTY_BLINK_RENDERER_PLATFORM_LOADER_FETCH_URL_LOADER_REQUEST_CONVERSION_H_
