# Building SSLBoard Desktop for macOS

This guide covers building the Tauri app for macOS, including code signing and notarization setup.

## Why Local Builds Don't Show This Error

If you build the app on your Mac locally (`npm run tauri build`), you won't see the "damaged" error. Here's why:

1. **No Quarantine Attribute**: When you download a file from the internet (like from GitHub Actions), macOS automatically adds a `com.apple.quarantine` extended attribute. This flag tells Gatekeeper to verify the app is signed/notarized. **Local builds don't have this attribute** because they weren't downloaded.

2. **Local Build Trust**: macOS is more lenient with apps built on the same machine. Even unsigned local builds often work because they're considered "trusted" by virtue of being created locally.

3. **Automatic Ad-Hoc Signing**: If you have Xcode or Command Line Tools installed, macOS might automatically perform ad-hoc signing during the build process, which is sufficient for local execution.

**The problem only appears when:**

- The app is downloaded from the internet (GitHub releases, email attachments, etc.)
- The app is transferred from another machine
- The app is distributed to other users

This is why code signing and notarization are essential for distribution, even though local builds work fine without them.

## Understanding Gatekeeper Messages

macOS Gatekeeper shows different messages depending on the app's signing status:

1. **"SSLBoard is damaged and can't be opened"**
   - App is **not code-signed**
   - Quick fix: Remove quarantine attribute (see below)

2. **"Apple could not verify 'SSLBoard' is free of malware"**
   - App is **code-signed but NOT notarized**
   - This is what you're seeing now - progress! The app is signed.
   - Solution: Notarize the app (see Notarization section below)

3. **No warning at all**
   - App is **code-signed AND notarized** ✅
   - This is the goal for distribution

## Quick Fix for "Damaged" Error

If you're getting the error **"SSLBoard is damaged and can't be opened"**, this is macOS Gatekeeper blocking an unsigned app. You have two options:

### Option 1: Remove Quarantine Attribute (Quick Test)

For local testing only, you can remove the quarantine attribute:

```bash
xattr -cr /path/to/SSLBoard.app
```

Or if you downloaded a DMG:

```bash
xattr -cr /path/to/SSLBoard.dmg
```

**Note:** This is only for testing. For distribution, you need proper code signing (see below).

### Option 2: Right-Click to Open (One-Time)

1. Right-click the app
2. Select "Open"
3. Click "Open" in the dialog

This tells macOS you trust the app, but it's not a permanent solution.

## "Apple could not verify" Error (Signed but Not Notarized)

If you see **"Apple could not verify 'SSLBoard' is free of malware"**, this means:

✅ **Good news**: Your app is code-signed!  
❌ **Issue**: It's not notarized yet.

**What this means:**

- The app has a valid code signature
- macOS 10.15+ (Catalina and later) requires notarization for distribution
- Users will see this warning until the app is notarized

**Quick Solutions:**

1. **For immediate testing**: Right-click the app → Select "Open" → Click "Open" in the dialog (works once per app)
2. **For distribution**: You **must** notarize the app (see sections below)

Notarization is required for macOS 10.15+ when distributing outside the App Store. Without it, users will see security warnings.

### Notarizing an Already-Built Release

If you've already built and released the app (e.g., from GitHub Actions) and it's not notarized, you can manually notarize it:

**Prerequisites:**

- Apple Developer account
- App-specific password (generate at [appleid.apple.com](https://appleid.apple.com))
- Your Team ID (from Apple Developer Portal)

**Steps:**

1. **Download the app bundle** from your release (extract from DMG if needed)

2. **Create a ZIP for notarization:**
   ```bash
   cd /path/to/SSLBoard.app/..
   ditto -c -k --keepParent SSLBoard.app SSLBoard.zip
   ```

3. **Submit for notarization:**
   ```bash
   xcrun notarytool submit SSLBoard.zip \
     --apple-id your@email.com \
     --team-id YOUR_TEAM_ID \
     --password "your-app-specific-password" \
     --wait
   ```

4. **Staple the notarization ticket:**
   ```bash
   xcrun stapler staple SSLBoard.app
   xcrun stapler validate SSLBoard.app
   ```

5. **Re-package and upload** the notarized app to your release

**Note:** If the app was built via GitHub Actions, ensure the required secrets are configured (see "Setting Up GitHub Actions" section below) so future builds are automatically notarized.

## Proper Solution: Code Signing & Notarization

For production builds that users can run without warnings, you need:

1. **Code Signing**: Sign the app with an Apple Developer certificate
2. **Notarization**: Submit the app to Apple for notarization (required for macOS 10.15+)

### Prerequisites

1. **Apple Developer Account** ($99/year)
   - Sign up at [developer.apple.com](https://developer.apple.com)

2. **Code Signing Certificate**
   
   To create a certificate, you first need to generate a Certificate Signing Request (CSR) from your Mac:
   
   **Generate a CSR file:**
   
   **Option A: Using Keychain Access (GUI)**
   
   1. Open **Keychain Access** (Applications → Utilities → Keychain Access)
   
   2. Go to **Keychain Access** → **Certificate Assistant** → **Request a Certificate From a Certificate Authority...**
   
   3. Fill in the form:
      - **User Email Address**: Your Apple ID email
      - **Common Name**: Your name or company name
      - **CA Email Address**: Leave empty
      - **Request is**: Select "Saved to disk"
   
   4. Click **Continue** and save the CSR file (e.g., `CertificateSigningRequest.certSigningRequest`)
   
   **Option B: Using Command Line**
   
   ```bash
   # Generate a private key and CSR in one command
   openssl req -new -newkey rsa:2048 -nodes \
     -keyout private_key.key \
     -out CertificateSigningRequest.certSigningRequest \
     -subj "/CN=Your Name/emailAddress=your@email.com"
   ```
   
   Note: If using the command line method, you'll need to import the private key into Keychain Access before exporting the `.p12` file later.
   
   **Request the certificate from Apple:**
   
   1. Go to [Apple Developer Portal - Certificates](https://developer.apple.com/account/resources/certificates/list)
   
   2. Click the **+** button to create a new certificate
   
   3. Select **Developer ID Application** (for distribution outside the App Store)
   
   4. Click **Continue**
   
   5. Upload the CSR file you just created
   
   6. Click **Continue** and then **Download** the certificate
   
   7. Double-click the downloaded `.cer` file to install it in your Keychain
   
   **Export as `.p12` file:**
   
   1. Open **Keychain Access**
   
   2. Find your certificate (look for "Developer ID Application: Your Name")
   
   3. Right-click the certificate and select **Export "Developer ID Application: Your Name"...**
   
   4. Choose **Personal Information Exchange (.p12)** format
   
   5. Set a password for the `.p12` file (you'll need this for GitHub Actions)
   
   6. Save the file (e.g., `certificate.p12`)

3. **App-Specific Password** (for notarization)
   - Generate at [appleid.apple.com](https://appleid.apple.com)
   - Go to "Sign-In and Security" → "App-Specific Passwords"
   - Create a password for "Xcode" or "Notarization"

### Setting Up GitHub Actions

The build workflow supports code signing when secrets are configured:

1. **Add GitHub Secrets** (Settings → Secrets and variables → Actions):

   - `APPLE_CERTIFICATE_BASE64`: Base64-encoded `.p12` certificate file
     ```bash
     base64 -i certificate.p12 | pbcopy
     ```
   
   - `APPLE_CERTIFICATE_PASSWORD`: Password for the `.p12` file
   
   - `APPLE_SIGNING_IDENTITY`: Signing identity (e.g., "Developer ID Application: Your Name (TEAM_ID)")
     ```bash
     security find-identity -v -p codesigning
     ```
   
   - `APPLE_TEAM_ID`: Your Apple Team ID (found in Apple Developer Portal)
   
   - `APPLE_ID`: Your Apple ID email
   
   - `APPLE_APP_SPECIFIC_PASSWORD`: App-specific password for notarization
   
   - `KEYCHAIN_PASSWORD`: Temporary password for the build keychain (can be any random string)

2. **Build will automatically:**
   - Import the certificate
   - Sign the app
   - Submit for notarization (if configured)
   - Create a release with the signed app

### Local Build with Code Signing

If building locally:

1. **Import certificate to keychain:**
   ```bash
   security import certificate.p12 -k ~/Library/Keychains/login.keychain
   ```

2. **Find your signing identity:**
   ```bash
   security find-identity -v -p codesigning
   ```

3. **Update `tauri.conf.json`:**
   ```json
   "bundle": {
     "macOS": {
       "signingIdentity": "Developer ID Application: Your Name (TEAM_ID)",
       "providerShortName": "TEAM_ID"
     }
   }
   ```

4. **Build:**
   ```bash
   npm run tauri build
   ```

5. **Notarize (after build):**
   ```bash
   xcrun notarytool submit SSLBoard.app \
     --apple-id your@email.com \
     --team-id TEAM_ID \
     --password "app-specific-password" \
     --wait
   ```

### Troubleshooting

#### "No signing identity found"

- Ensure the certificate is imported to your keychain
- Check that the signing identity matches exactly (including Team ID)
- Verify the certificate hasn't expired

#### "Notarization failed"

- Check Apple ID and app-specific password are correct
- Ensure the app is properly code-signed first
- Check notarization status: `xcrun notarytool history --apple-id your@email.com --team-id TEAM_ID`

#### "App still shows as damaged after signing"

- Notarization can take 5-30 minutes; wait and check status
- Ensure you're using a Developer ID certificate (not a Development certificate)
- Try stapling the notarization ticket: `xcrun stapler staple SSLBoard.app`

#### Verifying Code Signing and Notarization Status

To check if your app is properly signed and notarized:

```bash
# Check code signature
codesign -dv --verbose=4 /path/to/SSLBoard.app

# Verify code signature
codesign --verify --verbose /path/to/SSLBoard.app

# Check notarization status
spctl -a -vv /path/to/SSLBoard.app

# Check if notarization ticket is stapled
stapler validate /path/to/SSLBoard.app
```

**Expected output:**

- `codesign`: Should show "Developer ID Application: ..." and "valid on disk"
- `spctl`: Should show "accepted" and "source=Developer ID"
- `stapler validate`: Should show "The validate action worked!"

If `spctl` shows "rejected" or "no usable signature", the app needs notarization.

## Build Outputs

When you run `npm run tauri build`, Tauri will generate:

- **`.app` bundle**: `src-tauri/target/release/bundle/macos/SSLBoard.app`
  - Standalone application bundle
  - Can be distributed directly or packaged in a DMG

- **`.dmg`**: `src-tauri/target/release/bundle/dmg/SSLBoard_<version>_x64.dmg`
  - Disk image for distribution
  - Recommended for user-friendly installation

## Distribution Without Code Signing

If you don't have an Apple Developer account, you can:

1. Distribute the unsigned app with instructions to remove quarantine
2. Use a service like [MacUpdater](https://macupdater.net/) that handles signing
3. Consider open-source distribution platforms that accept unsigned apps

However, users will see security warnings, and some may be unable to run the app depending on their macOS security settings.

