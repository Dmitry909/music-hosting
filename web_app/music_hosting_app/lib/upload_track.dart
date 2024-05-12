import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:file_picker/file_picker.dart';
import 'package:dio/dio.dart';

import 'shared_state.dart';

class UploadTrackPage extends StatefulWidget {
  @override
  _UploadTrackPageState createState() => _UploadTrackPageState();
}

class _UploadTrackPageState extends State<UploadTrackPage> {
  TextEditingController _trackNameController = TextEditingController();
  String _selectFileStatus = 'File not selected';
  String _uploadTrackResult = '';
  PlatformFile? _selectedFile;

  Future<void> _selectFile() async {
    FilePickerResult? result = await FilePicker.platform.pickFiles();
    if (result != null) {
      setState(() {
        _selectedFile = result.files.single;
        _selectFileStatus = 'Selected file: ${_selectedFile?.path}';
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
    String pathToFile = _selectedFile?.path ?? "";
    if (pathToFile == "") {
      return;
    }

    final formData = FormData.fromMap({
      "file": await MultipartFile.fromFile(pathToFile, filename: filename),
      "track_name": trackName
    });
    final token = (await getToken())!;
    final headers = {'authorization': token, "Content-Type": "multipart"};

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
        // final body = response.body;
        // setState(() {_uploadTrackResult = 'Upload failed. response status code: $statusCode, body: $body';});
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
        title: Text('Upload track page'),
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            TextField(
              controller: _trackNameController,
              decoration: InputDecoration(
                labelText: 'Track name',
              ),
            ),
            SizedBox(height: 16),
            ElevatedButton(
              onPressed: _selectFile,
              child: Text('Select File'),
            ),
            Text(
              _selectFileStatus,
              style: TextStyle(
                color: _selectFileStatus == 'File not selected'
                    ? Colors.red
                    : Colors.green,
              ),
            ),
            SizedBox(height: 16),
            ElevatedButton(
              onPressed: _uploadTrack,
              child: Text('Upload track'),
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
