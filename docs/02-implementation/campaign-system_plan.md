# Implementation Plan: Advanced Campaign System & Real-Time Stats

## Goal
Enhance the Voice2Text Admin Dashboard with real-time statistics and a robust "Advanced Campaign Targeting" system. This system will allow administrators to filter clients based on criteria (e.g., license tier, activity duration) and instantly "Push" targeted messages to them. We will also improve the campaign creation UX with a rich editor.

## 1. Backend: Real-Time Statistics
**Current Status**: `api/admin/stats` exists and is fetching data from Supabase, but the Frontend `Dashboard` might need verification if data is empty.
**Action**:
- Verify `api/admin/stats` implementation covers all required metrics.
- Ensure `src/app/admin/page.tsx` correctly consumes this API.
- (Completed) Logic is already in place; mostly verification needed.

## 2. Backend: Campaign Infrastructure
**Current Status**: `v2t_campaigns` table exists in Supabase but was missing from Prisma. `POST /api/admin/campaigns/push` logic was partially drafted.
**Action**:
- [x] **Update Prisma Schema**: Added `Campaign` model to `schema.prisma`.
- [x] **Generate Prisma Client**: Run `prisma generate`.
- [ ] **Implement Push Logic**:
    - **Endpoint**: `POST /api/admin/campaigns/push`
    - **Input**: `{ campaignId }`
    - **Flow**:
        1. Fetch Campaign by ID.
        2. Parse `rule_json` (Targeting Rules).
        3. Query `Client` database for matches:
            - `tier` IN `rule_json.license_tiers`
            - `created_at` < `NOW - min_active_days`
            - `created_at` > `NOW - max_active_days`
        4. Generate `Message` records for each matching client.
        5. Bulk Insert into `v2t_messages`.
    - **Output**: JSON `{ success: true, count: <inserted_count> }`.

## 3. Frontend: Advanced Campaign Manager
**Current Status**: Basic CRUD with raw JSON inputs.
**Action**:
- **Refactor `src/app/admin/campaigns/page.tsx`**:
    - **Rich Editor UI**:
        - Replace JSON textareas with user-friendly inputs.
        - **Targeting Section**:
            - Checkboxes/Tags for License Tiers (Starter, Pro, Agency, Trial).
            - Number Input for "Minimum Days Active".
        - **Content Section**:
            - Text Input for "Title".
            - Textarea for "Body".
            - URL Input for "Action Link".
    - **Push Workflow**:
        - Add "PUSH" button to Campaign Cards.
        - Add Confirmation Dialog with "Dry Run" estimate (e.g., "This will message ~12 users").
        - Call Backend Push API.
        - Show Success/Error Toast.

## 4. Verification & Testing
- **Dry Run**: Create test clients with different tiers/dates in Supabase.
- **Create Campaign**: Use the new UI to create a campaign targeting "Pro" users active > 2 days.
- **Push**: Click Push -> Confirm.
- **Verify**: Check `v2t_messages` table to ensure only "Pro" users received the message.

## Tasks
1. [x] **Fix Schema**: Update `schema.prisma` and generate client.
2. [x] **Create Push API**: Implement `api/admin/campaigns/push`.
3. [x] **Refactor Frontend**: Rewrite `admin/campaigns/page.tsx` with new UI. (Code written, awaiting verification).
4. [ ] **Testing**: Verify flow end-to-end.
