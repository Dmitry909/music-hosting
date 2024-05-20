import 'package:flutter/material.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/foundation.dart' show kIsWeb;
import 'dart:typed_data';
import 'package:dio/dio.dart';

import 'shared_state.dart';

class UploadTrackPage extends StatefulWidget {
  const UploadTrackPage({super.key});

  @override
  _UploadTrackPageState createState() => _UploadTrackPageState();
}

class _UploadTrackPageState extends State<UploadTrackPage> {
  final TextEditingController _trackNameController = TextEditingController();
  String _selectFileStatus = 'File not selected';
  String _uploadTrackResult = '';
  PlatformFile? _selectedFile;

  Future<void> _selectFile() async {
    FilePickerResult? result = await FilePicker.platform.pickFiles();
    if (result != null) {
      setState(() {
        _selectedFile = result.files.single;
        _selectFileStatus = 'Selected file: ${_selectedFile?.name}';
      });
    }
  }

  Future<void> _uploadTrack() async {
    String trackName = _trackNameController.text;
    if (trackName == "") {
      setState(() {
        _uploadTrackResult = "Name not specified";
      });
      return;
    }

    if (_selectedFile == null) {
      setState(() {
        _uploadTrackResult = "File not selected";
      });
      return;
    }

    String filename = _selectedFile?.name ?? "";
    Uint8List? fileBytes;
    String? pathToFile;

    if (kIsWeb) {
      fileBytes = _selectedFile?.bytes!;
    } else {
      pathToFile = _selectedFile?.path ?? "";
      if (pathToFile == "") {
        return;
      }
    }

    final formData = FormData.fromMap({
      "track_name": trackName,
      "file": kIsWeb
          ? MultipartFile.fromBytes(fileBytes!, filename: filename)
          : await MultipartFile.fromFile(pathToFile!, filename: filename),
    });

    final token = (await getToken())!;
    final headers = {
      'authorization': token,
      "Content-Type": "multipart/form-data"
    };

    try {
      Response response = await Dio().post(
        'http://localhost:3000/upload_track',
        data: formData,
        options: Options(headers: headers),
      );

      if (response.statusCode == 201) {
        setState(() {
          _uploadTrackResult = 'Track uploaded';
          _selectFileStatus = 'File not selected';
          _trackNameController.clear();
        });
      } else {
        final statusCode = response.statusCode;
        setState(() {
          _uploadTrackResult =
              'Upload failed. response status code: $statusCode';
        });
      }
    } catch (e) {
      setState(() {
        _uploadTrackResult = 'Error occurred: $e. Please try again.';
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Upload track page'),
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            TextField(
              controller: _trackNameController,
              decoration: const InputDecoration(
                labelText: 'Track name',
              ),
            ),
            const SizedBox(height: 16),
            ElevatedButton(
              onPressed: _selectFile,
              child: const Text('Select File'),
            ),
            Text(
              _selectFileStatus,
              style: TextStyle(
                color: _selectFileStatus == 'File not selected'
                    ? Colors.red
                    : Colors.green,
              ),
            ),
            const SizedBox(height: 16),
            ElevatedButton(
              onPressed: _uploadTrack,
              child: const Text('Upload track'),
            ),
            Text(
              _uploadTrackResult,
              style: TextStyle(
                color: _uploadTrackResult == 'Track uploaded'
                    ? Colors.green
                    : Colors.red,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
