import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'package:flutter_passkey_service/flutter_passkey_service.dart';
import 'package:flutter_passkey_service/pigeons/messages.g.dart';
import 'package:zkool/main.dart';

/// Test screen for passkey PRF functionality
///
/// Correct PRF flow:
/// 1. "Register" (first time): authenticate() with PRF salt → derive seed
/// 2. "Auth" (later): authenticate() with SAME PRF salt → derive SAME seed
/// 3. Compare: Both should produce identical seeds
class PasskeyTestScreen extends StatefulWidget {
  const PasskeyTestScreen({super.key});

  @override
  State<PasskeyTestScreen> createState() => _PasskeyTestScreenState();
}

class _PasskeyTestScreenState extends State<PasskeyTestScreen> {
  final _rpId = 'hhanh00.github.io';
  final _rpName = 'zkool';
  String _status = 'Ready to test PRF determinism';
  String? _firstRecoveryCode;
  String? _secondRecoveryCode;
  bool _isLoading = false;
  final _scrollController = ScrollController();
  final _logMessages = <String>[];

  // Fixed salt for determinism testing (challenge can vary)
  final String _fixedSalt = base64Encode("salt".codeUnits);

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  void _addLog(String message) {
    setState(() {
      _logMessages.add('${DateTime.now()}: $message');
    });
    Future.delayed(const Duration(milliseconds: 100), () {
      if (_scrollController.hasClients) {
        _scrollController.animateTo(
          _scrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 300),
          curve: Curves.easeOut,
        );
      }
    });
  }

  Future<void> _registerWithPrf() async {
    try {
      _addLog('');
      _addLog('--- Step 1: First PRF Derivation ("Registration") ---');
      setState(() {
        _isLoading = true;
        _firstRecoveryCode = null;
        _secondRecoveryCode = null;
        _logMessages.clear();
      });

      // Generate or use fixed salt (challenge can vary)
      _addLog('RP ID: $_rpId');
      _addLog('RP Name: $_rpName');
      _addLog('PRF Key: "seed"');
      _addLog('PRF Salt: $_fixedSalt');

      // First, we need to register a passkey
      await _registerPasskey();

      // Then authenticate with PRF using the fixed challenge and salt
      await _deriveSeedWithPrf('first');

    } catch (e) {
      _addLog('❌ Error: $e');
      setState(() {
        _status = 'Registration failed: $e';
      });
    } finally {
      setState(() {
        _isLoading = false;
      });
    }
  }

  Future<void> _authenticateWithPrf() async {
    try {
      _addLog('');
      _addLog('--- Step 2: Second PRF Derivation ("Authentication") ---');

      setState(() {
        _isLoading = true;
      });

      // Derive seed with same salt
      await _deriveSeedWithPrf('second');

    } catch (e) {
      _addLog('❌ Error: $e');
      setState(() {
        _status = 'Authentication failed: $e';
      });
    } finally {
      setState(() {
        _isLoading = false;
      });
    }
  }

  Future<void> _registerPasskey() async {
    try {
      _addLog('');
      _addLog('--- Passkey Registration ---');

      // Generate challenge
      final challenge = _generateChallenge();
      final challengeBase64 = base64Url.encode(challenge);
      _addLog('Generated challenge: ${challengeBase64.substring(0, 20)}...');

      // Generate user ID
      final userId = 'hanh';
      final username = 'hanh';
      _addLog('User ID: $userId'); // account id
      _addLog('Username: $username');

      // Create registration options WITHOUT PRF (PRF is for authentication only)
      _addLog('Creating registration options (no PRF)...');
      final registrationOptions = FlutterPasskeyService.createRegistrationOptions(
        challenge: challengeBase64,
        rpName: _rpName,
        rpId: _rpId,
        userId: userId,
        username: username,
        displayName: 'PRF Test User',
        enablePrf: true,
      );
      _addLog('Registration options created');

      setState(() {
        _status = 'Complete the Face ID/Touch ID prompt for registration...';
      });
      _addLog('Waiting for biometric prompt...');

      // Register passkey
      _addLog('Calling passkey registration...');
      final response = await FlutterPasskeyService.register(registrationOptions);

      _addLog('✅ Passkey registered successfully!');
      _addLog('Credential ID: ${response.id.substring(0, 20)}...');

    } on PasskeyException catch (e) {
      _addLog('❌ Registration error: ${e.message} ${e.details}');
      logger.e(e.details);
      rethrow;
    }
  }

  Future<void> _deriveSeedWithPrf(String step) async {
    try {
      _addLog('');
      _addLog('--- PRF Seed Derivation ($step) ---');

      // Generate NEW challenge each time (allowed!)
      final challenge = _generateChallenge();
      final challengeBase64 = base64Url.encode(challenge);
      _addLog('Generated NEW challenge: ${challengeBase64.substring(0, 20)}...');
      _addLog('Using SAME salt: $_fixedSalt');
      _addLog('Challenge (base64): ${challengeBase64.substring(0, 20)}...');

      // Create authentication options WITH PRF
      _addLog('Creating authentication options WITH PRF...');
      _addLog('PRF eval: {"first": "$_fixedSalt"}');

      final authOptions = FlutterPasskeyService.createAuthenticationOptions(
        challenge: challengeBase64,
        rpId: _rpId,
        prfEval: {
          'first': _fixedSalt,
        },
      );
      _addLog('Authentication options created');

      setState(() {
        _status = 'Complete the Face ID/Touch ID prompt for PRF derivation...';
      });
      _addLog('Waiting for biometric prompt...');

      // Authenticate with PRF
      _addLog('Calling passkey authentication with PRF...');
      final response = await FlutterPasskeyService.authenticate(authOptions);

      _addLog('✅ Authentication successful!');
      _addLog('Checking for PRF results...');

      // Check PRF results
      final extensions = response.clientExtensionResults;
      if (extensions != null && extensions.prf != null) {
        _addLog('PRF extension available: YES');
        final prf = extensions.prf!;

        if (prf.results != null) {
          _addLog('PRF results: ${prf.results}');
          final results = prf.results!;

          if (results.containsKey('first')) {
            final derivedKey = results['first'];
            if (derivedKey != null && derivedKey.isNotEmpty) {
              _addLog('✅ PRF derived key found: ${derivedKey.substring(0, 20)}...');
              _addLog('Derived key length: ${derivedKey.length} chars');

              // Decode PRF output
              _addLog('Decoding PRF output from base64url...');
              final prfBytes = base64Url.decode(base64Url.normalize(derivedKey));
              _addLog('✅ PRF output decoded: ${prfBytes.length} bytes');
              _addLog('PRF bytes (hex): ${prfBytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join(' ')}');

              if (prfBytes.length == 32) {
                _addLog('✅ PRF output is correct length (32 bytes) - this is our seed!');

                // Create recovery code
                _addLog('Creating recovery code from PRF seed...');
                final recoveryCode = _encodeRecoveryCode(prfBytes);
                _addLog('✅ Recovery code created: $recoveryCode');

                if (step == 'first') {
                  setState(() {
                    _firstRecoveryCode = recoveryCode;
                    _status = '✅ Step 1 complete! Seed derived and stored.';
                  });
                  _addLog('💾 First recovery code stored: ${recoveryCode.substring(0, 20)}...');
                } else {
                  setState(() {
                    _secondRecoveryCode = recoveryCode;
                  });
                  _addLog('💾 Second recovery code created: ${recoveryCode.substring(0, 20)}...');

                  // Compare with first
                  if (_firstRecoveryCode != null) {
                    final match = _firstRecoveryCode == _secondRecoveryCode;
                    _addLog('🔍 Recovery code comparison: ${match ? "✅ MATCH" : "❌ DIFFERENT"}');
                    if (match) {
                      _addLog('✅ PRF IS DETERMINISTIC! Same salt = same seed = same recovery code!');
                      setState(() {
                        _status = '✅ DETERMINISTIC! PRF produces identical results.';
                      });
                    } else {
                      _addLog('❌ PRF produced different results with same salt!');
                      setState(() {
                        _status = '❌ NOT deterministic! Different results with same salt.';
                      });
                    }
                  }
                }

              } else {
                _addLog('❌ PRF output has wrong length: ${prfBytes.length} (expected 32)');
                setState(() {
                  _status = 'PRF output length: ${prfBytes.length} (expected 32)';
                });
              }
            } else {
              _addLog('❌ Derived key is empty');
              setState(() {
                _status = 'PRF derived key is empty';
              });
            }
          } else {
            _addLog('❌ No "seed" key in PRF results');
            _addLog('Available keys: ${results.keys.toList()}');
            setState(() {
              _status = 'No "seed" key in PRF results';
            });
          }
        } else {
          _addLog('❌ PRF results are null');
          setState(() {
            _status = 'PRF results are null';
          });
        }
      } else {
        _addLog('❌ PRF extension not available');
        setState(() {
          _status = 'PRF extension not available';
        });
      }

    } catch (e) {
      _addLog('❌ PRF derivation error: $e');
      setState(() {
        _status = 'PRF derivation failed: $e';
      });
    }
  }

  String _encodeRecoveryCode(Uint8List seed) {
    final hex = seed.map((b) => b.toRadixString(16).padLeft(2, '0')).join('');
    return 'zpass1$hex';
  }

  Uint8List _generateChallenge() {
    final random = Random.secure();
    return Uint8List.fromList(List<int>.generate(32, (_) => random.nextInt(256)));
  }

  void _clearLogs() {
    setState(() {
      _logMessages.clear();
      _firstRecoveryCode = null;
      _secondRecoveryCode = null;
      _status = 'Ready to test PRF determinism';
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('PRF Determinism Test'),
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
        actions: [
          IconButton(
            icon: const Icon(Icons.clear),
            onPressed: _clearLogs,
            tooltip: 'Clear logs',
          ),
        ],
      ),
      body: SingleChildScrollView(
        controller: _scrollController,
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            // Status card
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Status',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    const SizedBox(height: 8),
                    Text(_status),
                    if (_isLoading) ...[
                      const SizedBox(height: 16),
                      const LinearProgressIndicator(),
                    ],
                  ],
                ),
              ),
            ),

            const SizedBox(height: 16),

            // Test buttons
            Column(
              children: [
                ElevatedButton.icon(
                  onPressed: _isLoading ? null : _registerWithPrf,
                  icon: const Icon(Icons.face),
                  label: const Text('Step 1: Setup & Derive Seed'),
                  style: ElevatedButton.styleFrom(
                    padding: const EdgeInsets.all(16),
                  ),
                ),
                const SizedBox(height: 8),
                ElevatedButton.icon(
                  onPressed: (!_isLoading) ? _authenticateWithPrf : null,
                  icon: const Icon(Icons.lock),
                  label: const Text('Step 2: Verify Determinism'),
                  style: ElevatedButton.styleFrom(
                    padding: const EdgeInsets.all(16),
                  ),
                ),
              ],
            ),

            const SizedBox(height: 16),

            // Recovery codes display
            if (_firstRecoveryCode != null) ...[
              Card(
                color: Colors.blue.shade50,
                child: Padding(
                  padding: const EdgeInsets.all(16),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          Icon(Icons.numbers, color: Colors.blue.shade700),
                          const SizedBox(width: 8),
                          Text(
                            'First Recovery Code',
                            style: Theme.of(context).textTheme.titleMedium?.copyWith(
                              color: Colors.blue.shade700,
                            ),
                          ),
                        ],
                      ),
                      const SizedBox(height: 8),
                      Text(
                        _firstRecoveryCode!,
                        style: const TextStyle(
                          fontFamily: 'monospace',
                          fontSize: 14,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
              const SizedBox(height: 8),
            ],

            if (_secondRecoveryCode != null) ...[
              Card(
                color: Colors.green.shade50,
                child: Padding(
                  padding: const EdgeInsets.all(16),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          Icon(Icons.check_circle, color: Colors.green.shade700),
                          const SizedBox(width: 8),
                          Text(
                            'Second Recovery Code',
                            style: Theme.of(context).textTheme.titleMedium?.copyWith(
                              color: Colors.green.shade700,
                            ),
                          ),
                        ],
                      ),
                      const SizedBox(height: 8),
                      Text(
                        _secondRecoveryCode!,
                        style: const TextStyle(
                          fontFamily: 'monospace',
                          fontSize: 14,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
              const SizedBox(height: 16),
            ],

            // Logs
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        const Icon(Icons.terminal, size: 16),
                        const SizedBox(width: 8),
                        Text(
                          'Activity Log',
                          style: Theme.of(context).textTheme.titleMedium,
                        ),
                        const Spacer(),
                        if (_logMessages.isNotEmpty)
                          Text(
                            '${_logMessages.length} messages',
                            style: Theme.of(context).textTheme.bodySmall,
                          ),
                      ],
                    ),
                    const Divider(),
                    if (_logMessages.isEmpty)
                      const Padding(
                        padding: EdgeInsets.all(16),
                        child: Text('No activity yet'),
                      )
                    else
                      SizedBox(
                        height: 300,
                        child: ListView.builder(
                          shrinkWrap: true,
                          itemCount: _logMessages.length,
                          itemBuilder: (context, index) {
                            final message = _logMessages[index];
                            return Padding(
                              padding: const EdgeInsets.symmetric(vertical: 2),
                              child: Text(
                                message,
                                style: const TextStyle(
                                  fontFamily: 'monospace',
                                  fontSize: 11,
                                ),
                              ),
                            );
                          },
                        ),
                      ),
                  ],
                ),
              ),
            ),

            const SizedBox(height: 16),

            // Info
            Card(
              color: Colors.orange.shade50,
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Icon(Icons.info, color: Colors.orange.shade700),
                        const SizedBox(width: 8),
                        Text(
                          'How PRF Works',
                          style: Theme.of(context).textTheme.titleMedium?.copyWith(
                            color: Colors.orange.shade700,
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 8),
                    Text('RP ID: $_rpId'),
                    Text('PRF Key: "seed"'),
                    const SizedBox(height: 8),
                    Text(
                      '✅ Challenge can be different each time',
                      style: TextStyle(fontSize: 12, color: Colors.orange.shade700),
                    ),
                    Text(
                      '✅ Salt MUST be the same for determinism',
                      style: TextStyle(fontSize: 12, color: Colors.orange.shade700),
                    ),
                    Text(
                      'Same salt + same passkey = same seed (deterministic)',
                      style: TextStyle(fontSize: 12, color: Colors.orange.shade700),
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
