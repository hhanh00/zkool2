import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:file_picker/file_picker.dart';
import 'package:zkool/src/rust/api/contacts.dart';
import 'package:zkool/src/rust/api/openalias.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/validators.dart';
import 'package:zkool/widgets/scanner.dart';

class ContactEditPage extends ConsumerStatefulWidget {
  final Contact? contact;

  const ContactEditPage({super.key, this.contact});

  @override
  ConsumerState<ContactEditPage> createState() => ContactEditPageState();
}

class ContactEditPageState extends ConsumerState<ContactEditPage> {
  final _formKey = GlobalKey<FormBuilderState>();
  late var c = coinContext.coin;
  var _addresses = <String>[''];

  bool get isEditing => widget.contact != null;

  @override
  void initState() {
    super.initState();
    _addresses = widget.contact?.addresses.toList() ?? [''];
  }

  @override
  Widget build(BuildContext context) {
    return PopScope(
      canPop: false,
      onPopInvokedWithResult: (didPop, _) {
        if (didPop) return;
        onSaveAndPop();
      },
      child: Scaffold(
      appBar: AppBar(
        title: Text(isEditing ? "Edit Contact" : "New Contact"),
        actions: [
          IconButton(onPressed: onImport, tooltip: "Import from vCard", icon: Icon(Icons.download)),
          IconButton(onPressed: onExport, tooltip: "Export as vCard", icon: Icon(Icons.upload_file)),
          if (isEditing) IconButton(onPressed: onDelete, tooltip: "Delete contact", icon: Icon(Icons.delete)),
        ],
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.symmetric(horizontal: 12),
        child: FormBuilder(
          key: _formKey,
          child: Column(
            children: [
              Gap(16),
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(20),
                  child: Column(
                    children: [
                      FormBuilderTextField(
                        name: "name",
                        decoration: const InputDecoration(
                          labelText: "Name",
                          prefixIcon: Icon(Icons.person_outline),
                        ),
                        initialValue: widget.contact?.name ?? '',
                        validator: (v) =>
                            (v == null || v.trim().isEmpty) ? 'Name is required' : null,
                      ),
                    ],
                  ),
                ),
              ),
              Gap(12),
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(20),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          Text("Addresses",
                              style: Theme.of(context).textTheme.titleMedium),
                          IconButton(
                            icon: Icon(Icons.dns_outlined),
                            tooltip: "Import from OpenAlias",
                            onPressed: () => onImportOpenalias(),
                          ),
                        ],
                      ),
                      Gap(8),
                      ..._buildAddressFields(),
                    ],
                  ),
                ),
              ),
              Gap(12),
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(20),
                  child: Column(
                    children: [
                      FormBuilderTextField(
                        name: "notes",
                        decoration: const InputDecoration(
                          labelText: "Notes",
                          prefixIcon: Icon(Icons.notes),
                        ),
                        initialValue: widget.contact?.notes ?? '',
                        maxLines: 3,
                      ),
                    ],
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
      ),
    );
  }

  List<Widget> _buildAddressFields() {
    final widgets = <Widget>[];
    for (var i = 0; i < _addresses.length; i++) {
      widgets.add(
        Row(
          children: [
            Expanded(
              child: FormBuilderTextField(
                name: "address_$i",
                decoration: InputDecoration(
                  labelText: "Address ${i + 1}",
                  prefixIcon: Icon(Icons.qr_code),
                ),
                initialValue: _addresses[i],
                validator: (v) {
                  if (v == null || v.trim().isEmpty) return 'Address is required';
                  if (validAddress(v) != null) return 'Invalid address';
                  return null;
                },
              ),
            ),
            IconButton(
              icon: Icon(Icons.qr_code_scanner, color: Colors.blue),
              onPressed: () => onScanAddress(i),
              tooltip: "Scan QR code",
            ),
            if (i > 0)
              IconButton(
                icon: Icon(Icons.remove_circle_outline, color: Colors.red),
                onPressed: () => setState(() => _addresses.removeAt(i)),
                tooltip: "Remove address",
              ),
          ],
        ),
      );
      widgets.add(Gap(8));
    }
    widgets.add(
      Center(
        child: TextButton.icon(
          icon: Icon(Icons.add),
          label: Text("Add Address"),
          onPressed: () => setState(() => _addresses.add('')),
        ),
      ),
    );
    return widgets;
  }

  void onScanAddress(int index) async {
    final scanned = await showScanner(context, validator: validAddress);
    if (scanned != null) {
      setState(() {
        // Ensure we have the right number of items
        while (_addresses.length <= index) {
          _addresses.add('');
        }
        _addresses[index] = scanned;
        _formKey.currentState?.fields['address_$index']?.didChange(scanned);
      });
    }
  }

  void onImportOpenalias() async {
    print("Importing OpenAlias");
    final alias = await inputText(context, title: "Enter OpenAlias name");
    if (alias == null || alias.trim().isEmpty) return;

    try {
      final recipients = await resolveOpenalias(alias: alias.trim(), c: c);
      if (recipients.isEmpty) {
        showMessage(context, "No Zcash addresses were found for '$alias'.", title: "No address found");
        return;
      }
      // Fill empty slots first, then append remaining
      final newAddresses = List<String>.from(_addresses);
      for (final r in recipients) {
        final emptyIndex = newAddresses.indexWhere((a) => a.isEmpty);
        if (emptyIndex != -1) {
          newAddresses[emptyIndex] = r.address;
        } else {
          newAddresses.add(r.address);
        }
      }
      setState(() => _addresses = newAddresses);

      // Wait for widgets to rebuild before updating form fields
      WidgetsBinding.instance.addPostFrameCallback((_) {
        for (var i = 0; i < _addresses.length; i++) {
          _formKey.currentState?.fields['address_$i']?.didChange(_addresses[i]);
        }
      });
    } on AnyhowException catch (e) {
      showMessage(context, e.message, title: "OpenAlias Error");
    } catch (e) {
      showMessage(context, e.toString(), title: "OpenAlias Error");
    }
  }

  void onSave() async {
    if (!_formKey.currentState!.saveAndValidate()) return;

    final values = _formKey.currentState!.value;
    final name = (values['name'] as String).trim();
    final notes = (values['notes'] as String?)?.trim() ?? '';
    final addresses = <String>[];
    for (final key in values.keys) {
      if (key.startsWith('address_')) {
        final addr = (values[key] as String).trim();
        if (addr.isNotEmpty) addresses.add(addr);
      }
    }

    if (isEditing) {
      await updateContact(
        id: widget.contact!.id,
        name: name,
        addresses: addresses,
        notes: notes,
        c: c,
      );
    } else {
      await createContact(
        name: name,
        addresses: addresses,
        notes: notes,
        c: c,
      );
    }

    ref.invalidate(getContactsProvider);
  }

  void onSaveAndPop() async {
    if (!_formKey.currentState!.saveAndValidate()) return; // validation errors shown, block pop

    final values = _formKey.currentState!.value;
    final name = (values['name'] as String).trim();
    final notes = (values['notes'] as String?)?.trim() ?? '';
    final addresses = <String>[];
    for (final key in values.keys) {
      if (key.startsWith('address_')) {
        final addr = (values[key] as String).trim();
        if (addr.isNotEmpty) addresses.add(addr);
      }
    }

    if (isEditing) {
      await updateContact(id: widget.contact!.id, name: name, addresses: addresses, notes: notes, c: c);
    } else {
      await createContact(name: name, addresses: addresses, notes: notes, c: c);
    }

    ref.invalidate(getContactsProvider);
    if (mounted) GoRouter.of(context).pop();
  }

  void onDelete() async {
    final confirmed = await confirmDialog(
      context,
      title: "Delete contact?",
      message: "'${widget.contact!.name}' will be permanently deleted.",
    );
    if (confirmed) {
      await deleteContacts(ids: [widget.contact!.id], c: c);
      ref.invalidate(getContactsProvider);
      if (mounted) GoRouter.of(context).pop();
    }
  }

  void onImport() async {
    final result = await FilePicker.platform.pickFiles(
      type: FileType.any,
      allowMultiple: false,
    );
    if (result != null && result.files.single.path != null) {
      final data = await File(result.files.single.path!).readAsString();
      final contacts = await importContactsVcard(vcardData: data, c: c);
      if (contacts.isNotEmpty) {
        final contact = contacts.first;
        setState(() {
          final form = _formKey.currentState!;
          form.fields['name']!.didChange(contact.name);
          form.fields['notes']!.didChange(contact.notes);
          _addresses = contact.addresses.isEmpty ? [''] : contact.addresses.toList();
        });
      }
      ref.invalidate(getContactsProvider);
    }
  }

  void onExport() async {
    final form = _formKey.currentState!;
    form.saveAndValidate();
    final values = form.value;
    final name = (values['name'] as String?)?.trim() ?? '';
    final notes = (values['notes'] as String?)?.trim() ?? '';
    final addresses = <String>[];
    for (final key in values.keys) {
      if (key.startsWith('address_')) {
        final addr = (values[key] as String).trim();
        if (addr.isNotEmpty) addresses.add(addr);
      }
    }

    // Build note with addresses as zcash:<addr> payment URIs
    final noteLines = <String>[
      if (notes.isNotEmpty) notes,
      if (addresses.isNotEmpty) 'Zcash addresses:\n${addresses.map((a) => 'zcash:$a').join('\n')}',
    ];
    final note = noteLines.join('\n\n');

    final buffer = StringBuffer();
    buffer.writeln('BEGIN:VCARD');
    buffer.writeln('VERSION:4.0');
    buffer.writeln('FN:$name');
    if (note.isNotEmpty) buffer.writeln('NOTE:$note');
    buffer.writeln('END:VCARD');

    await saveFile(
      title: 'Export Contact',
      fileName: '${name.replaceAll(RegExp(r'[^a-zA-Z0-9]'), '_')}.vcf',
      data: Uint8List.fromList(buffer.toString().codeUnits),
    );
  }
}
